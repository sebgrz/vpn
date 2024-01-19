# VPN via WS - proof of concept

## Build
Just:  
`cargo build`  
or
`cargo build --release`  

## Setup
### Server
Start server by command:  
`./server <port>`


### Clients
Client start up command is:  
`sudo ./client ws://<server_ip>:<port>`

Sudo is required because it's create a network interface vpn with a name: vpn0.

### Networking
Next step is to asssign ip addresses to the clients:
```
# client 1
sudo ip address add 10.0.2.2/24 dev vpn0
sudo ip link set dev vpn0 up

# client 2
sudo ip address add 10.0.2.3/24 dev vpn0
sudo ip link set dev vpn0 up
```

That is enough to connect two devices via vpn via websockets.
```
# from client 1
ping 10.0.2.3

# from client 2
ping 10.0.2.2
```

## Test with http server
> Requirements:
>
> nodejs and npm

Let's create simple html page on the client 2 (10.0.2.3):  
`echo "test vpn" >> web/index.html`

and then expose it via simple http server from node:  
`npx serve -l 54321 web/`

So, right now should be possible to curl from client 1 to client2 on port 54321 and receive content of the index.html file.  
`curl http://10.0.2.3:54321`

## Internal network
Let's assume - client 1 want to access to the internal network of client2 (10.0.0.0/24).  
To do this **client 1** needs to set up routing **10.0.0.0/24** subnet via **10.0.2.1 (vpn) gateway**:  
`sudo ip route add 10.0.0.0/24 via 10.0.2.1 dev vpn0`

**client 2** should forward packets from vpn0 interface to the internal network interface:  
`sudo iptables -A FORWARD -i vpn0 -j ACCEPT;sudo iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE`

