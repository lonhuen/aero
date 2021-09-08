import collections
import functools
from tensorflow_model_optimization.python.core.internal import tensor_encoding as te

import sys
sys.path.append('../')
import time
# import model

# import tensorflow.compat.v1 as tf

import numpy as np
import tensorflow as tf
import tensorflow_federated as tff
from tensorflow_privacy.privacy.analysis import compute_dp_sgd_privacy
from tensorflow_privacy.privacy.optimizers.dp_optimizer import DPGradientDescentGaussianOptimizer
from tensorflow.keras.layers import Dense, Dropout, Flatten, Conv2D, MaxPool2D, BatchNormalization
import time

# np.random.seed(0)

# This value only applies to EMNIST dataset, consider choosing appropriate
# values if switching to other datasets.
MAX_CLIENT_DATASET_SIZE = 418
CLIENT_EPOCHS_PER_ROUND = 5
CLIENT_BATCH_SIZE = 16
TEST_BATCH_SIZE = 500
NUM_ROUNDS = 20
NUM_CLIENTS_PER_ROUND = 50

emnist_train, emnist_test = tff.simulation.datasets.emnist.load_data(
    only_digits=True)


def reshape_emnist_element(element):
  return (tf.expand_dims(element['pixels'], axis=-1), element['label'])


def preprocess_train_dataset(dataset):
  """Preprocessing function for the EMNIST training dataset."""
  return (dataset
          # Shuffle according to the largest client dataset
          .shuffle(buffer_size=MAX_CLIENT_DATASET_SIZE)
          # Repeat to do multiple local epochs
          .repeat(CLIENT_EPOCHS_PER_ROUND)
          # Batch to a fixed client batch size
          .batch(CLIENT_BATCH_SIZE, drop_remainder=False)
          # Preprocessing step
          .map(reshape_emnist_element))

def preprocess_test_dataset(dataset):
  return dataset.map(reshape_emnist_element)
  # .batch(
      # CLIENT_BATCH_SIZE, drop_remainder=False)


emnist_train = emnist_train.preprocess(preprocess_train_dataset)


def create_keras_model(input_shape):
  """The CNN model used in https://arxiv.org/abs/1602.05629."""
  model = tf.keras.models.Sequential([
      tf.keras.layers.InputLayer(input_shape=(28, 28, 1)),
      Conv2D(96,(11,11), activation="relu"),
			BatchNormalization(axis = 3),
			MaxPool2D((3,3), strides = 2),
			Conv2D(256,(5,5), activation="relu", padding = 'same'),
			BatchNormalization(axis = 3),
			MaxPool2D((3,3),strides = 2),
			Conv2D(384, (3,3) , activation="relu", padding = 'same'),
			BatchNormalization(axis = 3),
			Conv2D(384, (3,3) , activation="relu", padding = 'same'),
			BatchNormalization(axis = 3),
			Conv2D(256, (3,3) , activation="relu", padding = 'same'),
			BatchNormalization(axis = 3),
			MaxPool2D((3,3),strides = 2),
			Flatten(),
			Dense(4096, activation = 'relu'),
			Dense(4096, activation = 'relu'),
			Dense(10),
      tf.keras.layers.Softmax(),
  ])

  print(model.summary())

  return model


# Gets the type information of the input data. TFF is a strongly typed
# functional programming framework, and needs type information about inputs to 
# the model.
input_spec = emnist_train.create_tf_dataset_for_client(
    emnist_train.client_ids[0]).element_spec

def make_federated_data(client_data, client_ids):
  return [
      # client_data.preprocess(preprocess_train_dataset)
      preprocess_train_dataset(client_data.create_tf_dataset_for_client(x))
      for x in client_ids
  ]


def model_fn():
  # We _must_ create a new model here, and _not_ capture it from an external
  # scope. TFF will call this within different graph contexts.
  keras_model = create_keras_model((28,28,1))
  # keras_model = model.cifar_model_baseline((28,28,1))
  return tff.learning.from_keras_model(
      keras_model=keras_model,
      input_spec=input_spec,
      loss=tf.keras.losses.SparseCategoricalCrossentropy(),
      metrics=[tf.keras.metrics.SparseCategoricalAccuracy()])


fed_avg = tff.learning.build_federated_averaging_process(
    model_fn=model_fn,
    client_optimizer_fn=lambda: tf.keras.optimizers.SGD(learning_rate=0.02),
    server_optimizer_fn=lambda: tf.keras.optimizers.SGD(learning_rate=1.0))


def train(federated_averaging_process, num_rounds, num_clients_per_round, summary_writer):
  """Trains the federated averaging process and output metrics."""
  # Create a environment to get communication cost.
  environment = tff.framework.sizing_executor_factory()
  print(environment)
  # env = environment.create_executor(None)
  # environment = set_sizing_environment()

  # Initialize the Federated Averaging algorithm to get the initial server state.
  state = federated_averaging_process.initialize()

  with summary_writer.as_default():
    for round_num in range(num_rounds):
      # Sample the clients parcitipated in this round.
      sampled_clients = np.random.choice(
          emnist_train.client_ids,
          size=num_clients_per_round,
          replace=False)
      # Create a list of `tf.Dataset` instances from the data of sampled clients.
      sampled_train_data = [
          emnist_train.create_tf_dataset_for_client(client)
          for client in sampled_clients
      ]
      # Round one round of the algorithm based on the server state and client data
      # and output the new state and metrics.
      
      start = time.time()
      state, metrics = federated_averaging_process.next(state, sampled_train_data)
      end = time.time()
      print("Time to train 1 client's data is: ", end-start)


      # For more about size_info, please see https://www.tensorflow.org/federated/api_docs/python/tff/framework/SizeInfo
      size_info = environment.get_size_info()
      print(size_info)
      broadcasted_bits = 0# size_info.broadcast_bits[-1]
      aggregated_bits = 0#size_info.aggregate_bits[-1]

      print('round {:2d}, metrics={}, broadcasted_bits={}, aggregated_bits={}'.format(round_num, metrics, broadcasted_bits, aggregated_bits))

      # Add metrics to Tensorboard.
      for name, value in metrics['train'].items():
          tf.summary.scalar(name, value, step=round_num)

      # Add broadcasted and aggregated data size to Tensorboard.
      tf.summary.scalar('cumulative_broadcasted_bits', broadcasted_bits, step=round_num)
      tf.summary.scalar('cumulative_aggregated_bits', aggregated_bits, step=round_num)
      summary_writer.flush()

  return state

# Clean the log directory to avoid conflicts.
# !rm -R /tmp/logs/scalars/*

# Set up the log directory and writer for Tensorboard.
logdir = "/tmp/logs/scalars/original/"
summary_writer = tf.summary.create_file_writer(logdir)

final_state = train(federated_averaging_process=fed_avg, num_rounds=NUM_ROUNDS,
      num_clients_per_round=NUM_CLIENTS_PER_ROUND, summary_writer=summary_writer)
print(type(final_state))

emnist_test_preprocessed = preprocess_test_dataset(
 emnist_test.create_tf_dataset_from_all_clients().batch(128)
)

model = create_keras_model((28,28,1))
model.compile(optimizer=tf.keras.optimizers.SGD(learning_rate=0.01),
                       loss=tf.keras.losses.SparseCategoricalCrossentropy(),
                       metrics=[tf.keras.metrics.SparseCategoricalAccuracy()])
final_state.model.assign_weights_to(model)

loss, accuracy = model.evaluate(emnist_test_preprocessed)
print('\tEval: loss={l:.3f}, accuracy={a:.3f}'.format(l=loss, a=accuracy))


sample_clients = np.random.choice(
        emnist_train.client_ids,
        size=500,
        replace=False)
fed_test_data = make_federated_data(emnist_test, sample_clients)
evaluation = tff.learning.build_federated_evaluation(model_fn)
test_metrics = evaluation(final_state.model, fed_test_data)
print("Test metrics:", str(test_metrics))
