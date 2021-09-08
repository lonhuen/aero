import collections
import functools
from tensorflow_model_optimization.python.core.internal import tensor_encoding as te

import numpy as np
import tensorflow as tf
import tensorflow_federated as tff
import time 


'''
DNN with EMNIST dataset
'''

np.random.seed(0)
emnist_train, emnist_test = tff.simulation.datasets.emnist.load_data()

NUM_CLIENTS = 50
NUM_EPOCHS = 5
BATCH_SIZE = 32
SHUFFLE_BUFFER = 100
PREFETCH_BUFFER = 10
NUM_ROUNDS = 20


def preprocess(dataset):

  def batch_format_fn(element):
    """Flatten a batch `pixels` and return the features as an `OrderedDict`."""
    return collections.OrderedDict(
        x=tf.reshape(element['pixels'], [-1, 784]),
        y=tf.reshape(element['label'], [-1, 1]))

  return dataset.repeat(NUM_EPOCHS).shuffle(SHUFFLE_BUFFER).batch(
      BATCH_SIZE).map(batch_format_fn).prefetch(PREFETCH_BUFFER)


example_dataset = emnist_train.create_tf_dataset_for_client(
    emnist_train.client_ids[0])
example_element = next(iter(example_dataset))
preprocessed_example_dataset = preprocess(example_dataset)
sample_batch = tf.nest.map_structure(lambda x: x.numpy(),
                                     next(iter(preprocessed_example_dataset)))


def make_federated_data(client_data, client_ids):
  return [
      preprocess(client_data.create_tf_dataset_for_client(x))
      for x in client_ids
  ]


def create_keras_model():
  return tf.keras.models.Sequential([
      tf.keras.layers.Input(shape=(784,)),
      tf.keras.layers.Dense(10, kernel_initializer='zeros'),
      tf.keras.layers.Softmax(),
  ])


def model_fn():
  # We _must_ create a new model here, and _not_ capture it from an external
  # scope. TFF will call this within different graph contexts.
  keras_model = create_keras_model()
  return tff.learning.from_keras_model(
      keras_model,
      input_spec=preprocessed_example_dataset.element_spec,
      loss=tf.keras.losses.SparseCategoricalCrossentropy(),
      metrics=[tf.keras.metrics.SparseCategoricalAccuracy()])


iterative_process = tff.learning.build_federated_averaging_process(
    model_fn,
    client_optimizer_fn=lambda: tf.keras.optimizers.SGD(learning_rate=0.01),
    server_optimizer_fn=lambda: tf.keras.optimizers.SGD(learning_rate=1.0))

environment = tff.framework.sizing_executor_factory()
print(environment)
state = iterative_process.initialize()

for round_num in range(0, NUM_ROUNDS):
  sample_clients = np.random.choice(
        emnist_train.client_ids,
        size=NUM_CLIENTS,
        replace=False)
  
  federated_train_data = make_federated_data(emnist_train, sample_clients)
  
  size_info = environment.get_size_info()
  print(size_info)
  
#   start = time.time()
  state, metrics = iterative_process.next(state, federated_train_data)
#   end = time.time()
#   print("Time to train 1 client's data is: ", end-start)

  print('round {:2d}, metrics={}'.format(round_num, metrics))


evaluation = tff.learning.build_federated_evaluation(model_fn)
train_metrics = evaluation(state.model, federated_train_data)
print("Train metrics:", str(train_metrics))

np.random.choice(emnist_test.client_ids,
        size=500,
        replace=False)
# sample_clients = emnist_test.client_ids

federated_test_data = make_federated_data(emnist_test, sample_clients)
test_metrics = evaluation(state.model, federated_test_data)
print("Test metrics:", str(test_metrics))
