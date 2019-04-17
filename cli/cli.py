import socket
import sys
import requests
import random
import json
from time import time
import six


def connect(ip: str, port: int):
    s = socket.socket()
    s.connect((ip, int(port)))
    return s


def encode_varint(number):
    parts = list()
    length = 0
    while True:
        if not length:
            e = 0x00
        else:
            e = 0x80
        parts.append(six.int2byte((number & 0x7f) | e))
        if number <= 0x7f:
            break
        number = (number >> 7) - 1
        length += 1
    return b''.join(reversed(parts))


# Grab RPC server address
rpc_addr = sys.argv[1]
rpc_ip, _, rpc_port = rpc_addr.rpartition(":")

# Grab command
cmd = sys.argv[2]

if cmd == "addpeer":
    # Grab server address
    server_addr = sys.argv[3]
    ip, seperator, port = server_addr.rpartition(":")

    # Create add peer message
    ip_bytes = bytes(map(int, ip.split(".")))
    msg = b"\x00" + ip_bytes + int(port).to_bytes(2, byteorder="big")

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
        ip_bytes = bytes(map(int, server_ip.split(".")))
        msg = b"\x00" + ip_bytes + int(8332).to_bytes(2, byteorder="big")

        # Send message
        connect(rpc_ip, rpc_port).send(msg)

elif cmd == "newtransaction":
    # Load binary
    aux = bytes(sys.argv[3], "utf8")
    binary_path = sys.argv[4]

    with open(binary_path, "rb") as f:
        binary = f.read()

    # Create transaction
    time_vi = encode_varint(int(time() * 1_000))
    aux_len = encode_varint(len(aux))
    bin_len = encode_varint(len(binary))
    msg = b"\x01" + time_vi + aux_len + aux + bin_len + binary

    connect(rpc_ip, rpc_port).send(msg)
