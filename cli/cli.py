import socket
import sys
import requests
import random
import json

def connect(ip: str, port: int):
    s = socket.socket()
    s.connect((ip, int(port)))
    return s

# Grab RPC server address
rpc_addr = sys.argv[1]
rpc_ip, _, rpc_port = rpc_addr.rpartition(':')

# Grab command
cmd = sys.argv[2]

if cmd == "addpeer":
    # Grab server address
    server_addr = sys.argv[3]
    ip, seperator, port = server_addr.rpartition(':')

    # Create add peer message
    ip_bytes = bytes(map(int, ip.split('.')))
    msg = b"\x00" + ip_bytes + int(port).to_bytes(2, byteorder='big')

    # Send message
    connect(rpc_ip, rpc_port).send(msg)

elif cmd == "discover":
    # Get hosts from DNS
    resp = requests.get("http://discover.cauchyledger.io/")
    hosts = json.loads(resp.content)["hosts"]
    print("Found {} potential hosts".format(len(hosts)))

    # Grab server address
    n_peers = int(sys.argv[3])
    chosen_hosts = random.choices(hosts, k=n_peers)
    
    for server_ip in chosen_hosts:
        # Create add peer message
        ip_bytes = bytes(map(int, server_ip.split('.')))
        msg = b"\x00" + ip_bytes + int(8332).to_bytes(2, byteorder='big')

        # Send message
        connect(rpc_ip, rpc_port).send(msg)