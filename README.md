<div align="center">

# Secure share [![Release](https://github.com/Onboardbase/secure-share/actions/workflows/release.yml/badge.svg)](https://github.com/Onboardbase/secure-share/actions/workflows/release.yml)[![Lint](https://github.com/Onboardbase/secure-share/actions/workflows/lint.yml/badge.svg)](https://github.com/Onboardbase/secure-share/actions/workflows/lint.yml)

Share anything with teammates across machines via CLI. With Share, you can send something P2P, so you don't have to store whatever you want to share on somebody else's server. You don't have to set up a new server to transfer files to a teammate.
</div>

# Contents

- [Dependencies](#dependencies)
- [Install](#install)
- [Usage](#usage)
  - [Files](#files)
  - [Messages](#messages)
  - [Configuration](#configuration)
- [Update](#update)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Technical Details](#techincals)

## Dependencies

- `bash`, `curl`, `tar`: install these utilities.

## Install 
To use `share`,
```bash
yarn add @onboardbase/secure-share # npm i @onboardbase/secure-share
```
Or, using curl:
```sh
curl https://wokebuild.github.io | bash
```
Notes:
- For Windows users, please use `Git Bash` or any other CLI with the Bourne Shell.

and then,
```shell
share --help # ./share --help if you used the bash script
```
You should get a response displaying the utilities for `share`
```
Share anything with teammates across machines via CLI.

Usage: share [OPTIONS] <MODE>

Arguments:
  <MODE>  The mode (send secrets, or receive secrets). e,g `share send` or `share receive`
Options:
  -s, --secret <SECRET>
          Separated list of secrets to share. The key-Value pair is separated by a comma. "my_key,my_value"
  -m, --message <MESSAGE>
          List of messages or a message string to deliver to the receiver. e,g -m "Hi there" -m "See me"
  -f, --file <FILE>
          List of file paths of files to deliver to the receiver. e,g -f "/path/to/file1" -f "../path/to/file2"
  -r, --remote-peer-id <REMOTE_PEER_ID>
          Peer ID of the remote to send secrets to
  -p, --port <PORT>
          Port to establish a connection on
  -d, --debug...
          Turn debugging information on
  -h, --help
          Print help
  -V, --version
          Print version
```


## Usage
`share` enables the transmission of secrets or messages between teammates using different machines and behind different networks. To share a secret, the sender and receiver must both get `share` as described above and follow the instructions below.

#### The receiver:
Open a terminal or `cd` to where `share` was installed, then:
```shell
share receive
```
`share` starts in listen mode and assigns you a `PeerId`, and picks a random port to start on. (An optional `-p` flag is available to specify a port). A response like the one below should be displayed:
```
INFO  Your PeerId is: 12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt
INFO  Listening on "/ip4/172.19.192.1/tcp/54654"
INFO  Listening on "/ip4/192.168.0.197/tcp/54654"
INFO  Listening on "/ip4/127.0.0.1/tcp/54654"
INFO  Listening on "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt"
```

#### The sender:
Obtain the `PeerId` of the teammate you wish to send a secret to, then:
```shell
share send -r 12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt -s "hello, woke"
```
`share` will print your IP address and your `PeerId`.
To verify that a connection was established and your machine can talk to your teammates, you should see a similar thing below in your terminal:
```
INFO  Your PeerId is: 12D3KooWRpqX3QUvPNHXW5utkceLbx2b1LKfuAKa3iLdXXBGB2bY
INFO  Listening on "/ip4/127.0.0.1/tcp/40479"
INFO  Listening on "/ip4/192.168.212.254/tcp/40479"
INFO  Established connection to 12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt via /ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/12D3KooWA768LzHMatxkjD1f9DrYW375GZJr6MHPCNEdDtHeTNRt
```

The sender then attempts to send the secret, and if it is successful, `share` relays  messages to both parties, notifying them of the status and the progress of the secret sharing session.

  ## Files
  `share` also supports sending files:
  ```shell
  share send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -f ../path/to/file1 -f path/to/file2
  ```
  ## Messages
  Ordinary messages can also be shared
  ```shell
  share send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -m "hi there" -m "foo"
  ```
  All three items can also be sent together.

  ## Configuration
  As of `v0.0.12`, `share` allows a configuration file to be passed. Ports, whitelists, and items can all be configured directly instead of passing them as arguments. A sample configuration file can be found [here](./config.yml). For example:

  ```shell
  share receive -c ./config.yml
  ```
  Or for senders:

  ```shell
  share send -r 12D3KooWLaLnHjKhQmB46jweVXCDKVy4AL58a4S4ZgHZGuJkzBf9 -c ./config.yml
  ```


# Contributing

Contributions of any kind are welcome! See the [contributing guide](contributing.md).

[Thanks goes to these contributors](https://github.com/Onboardbase/secure-share/graphs/contributors)!

# Roadmap

### Utilities
- [ ] Configuration File: Enables users to pass in a config file as an argument instead of listing all parameters manually.
  - [x] Default path to save items(messgaes, secrets and files).
  - [x] Replace secrets or update them
  - [x] When files with the same name are received, discard, keep, inform, or update them
  - [ ] Add a whitelist of IPs to allow connection from

### Security
- [ ] Signed Certificates from Let's Encrypt.
- [ ] Whitelists and Blacklists

### Protocols
- [ ] Support QUIC. Use QUIC as default and fall back to TCP
- [ ] AutoNat: If you look closely, `share` assumes both peers are behind NATs, firewalls, or proxies. But sometimes, this might not be the case, and it is excessive to hole punch just for that. Implementing `AutoNat` will first check if the two peers can communicate directly. If not, it will then proceed to hole punch. With TCP, this might take about 3 to 10 seconds, and this is where QUIC comes in and improves upon `share`'s speed.

### Miscellaneous
- [ ] Send via disposable tunnel links. Create a tunnel link from the secret, and send the URL to the receiver. Once the sharing is done, you can close the tunnel, and the URL becomes unavailable.
- [ ] Curl command to an API endpoint without local download.
- [ ] Personalize Peer ID + Allow saving recipient info (address, port, etc.) and giving a proper name so one can do "share send dante -m Hello"
- [ ] Allow the possibility to always listen to specific addresses so that there can be a free flow of data
- [ ] Teams

# License

See [LICENSE](LICENSE) © [Onboardbase](https://github.com/Onboardbase/)

# Technicals

The major technical detail `share` employs under the hood is P2P sharing. Below are excellent and detailed resources on P2P sharing and hole punching. Happy reading!!
  - https://blog.ipfs.tech/2022-01-20-libp2p-hole-punching/
  - https://tailscale.com/blog/how-nat-traversal-works/
