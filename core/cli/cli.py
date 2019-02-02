import socket
import sys

server_addr = sys.argv[1]
ip, _, port = server_addr.rpartition(':')
s = socket.socket()
s.connect((ip, int(port)))

cmd = sys.argv[2]
args = sys.argv[3]
if cmd == "addpeer":
    ip, seperator, port = args.rpartition(':')
    ip_bytes = bytes(map(int, ip.split('.')))
    msg = b"\x00" + ip_bytes + int(port).to_bytes(2, byteorder='big')
    s.send(msg)
# elif cmd == "handshakepeer":
#     ip, seperator, port = args.rpartition(':')
#     ip_bytes = bytes(map(int, ip.split('.')))
#     msg = b"\x00" + ip_bytes + int(port).to_bytes(2, byteorder='big')
#     s.send(msg)

    