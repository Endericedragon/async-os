import socket

s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s.connect(("127.0.0.1", 7878))
print(s.send(b"Hello!"))
data = s.recv(1024)
print(data)
s.close()
