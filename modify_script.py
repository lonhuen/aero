import configparser
import json
config = configparser.ConfigParser()
with open('config.ini', 'r') as f:
        config_string = '[Atom]\n' + f.read()
        config.read_string(config_string)
#config.read('config.ini')
# print(config["Atom"]["real_worker_addr"])
#l = json.loads(config["Atom"]["real_worker_addr"])
# print(l[0])
# print(config["Atom"]["nr_real_worker"])
nr_real = int(config["Atom"]["nr_real"])
nr_simulated = int(config["Atom"]["nr_simulated"])
nr_sybil = int(config["Atom"]["nr_sybil"])

real_addr = json.loads(config["Atom"]["real_worker_addr"])
nr_real_per_worker = [int(x) for x in json.loads(
    config["Atom"]["nr_real_per_worker"])]

sim_addr = json.loads(config["Atom"]["sim_worker_addr"])
nr_sim_per_worker = [int(x) for x in json.loads(
    config["Atom"]["nr_sim_per_worker"])]

for w in real_addr:
    print("scp -i ./data/aws01.pem config.ini ubuntu@{}:".format(w))
for w in sim_addr:
    print("scp -i ./data/aws01.pem config.ini ubuntu@{}:".format(w))

# configure the network
for w in real_addr:
    print('ssh -i ./data/aws01.pem ubuntu@{} "bash ~/quail/scripts/network.sh" &'.format(w,))

# start the server
print("cargo run --bin server --release &")

# config the network
for w in real_addr:
    print('ssh -i ./data/aws01.pem ubuntu@{} "bash ~/quail/scripts/network.sh" &'.format(w,))

# run the clients
for i in range(len(real_addr)):
    print('ssh -i ./data/aws01.pem ubuntu@{} ~/quail/scripts/exp.sh {} &'.format(
        real_addr[i], nr_real_per_worker[i]))

# run the simluated clients
if len(sim_addr) > 0:
    print('ssh -i ./data/aws01.pem ubuntu@{} "cargo run --bin light_client --release" {} &'.format(
        sim_addr[0], nr_sim_per_worker[0]))
print("wait")
print("sudo pkill -P $$")
