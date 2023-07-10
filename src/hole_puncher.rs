//This "Direct Connection Upgrade through Relay" (DCUTR)  allows peers to establish direct connections with each other.
//i.e hole punching

use std::{error::Error, process::exit};

use crate::{
    secret::{Secret, SecretResponse},
    Mode,
};
use futures::{
    executor::{block_on, ThreadPool},
    stream::StreamExt,
    FutureExt,
};
use libp2p::{
    core::{muxing::StreamMuxerBox, upgrade},
    dcutr,
    dns::DnsConfig,
    identify, identity,
    multiaddr::Protocol,
    noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, StreamProtocol, Transport,
};
use rand::Rng;
use request_response::{self, json, ProtocolSupport};
use tracing::{error, info};

use super::Cli;

pub fn punch(opts: Cli) -> Result<(), Box<dyn Error>> {
    let relay_address: Multiaddr =
        "/ip4/157.245.40.97/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN"
            .to_string()
            .parse()
            .unwrap();
    let secret_key_seed = rand::thread_rng().gen_range(0..100);
    let port = opts.port.unwrap_or(0).to_string();

    let local_key = generate_ed25519(secret_key_seed);
    let local_peer_id = PeerId::from(local_key.public());
    info!("Local peer id: {:?}", local_peer_id);

    //intitate relay client connection
    let (relay_transport, client) = relay::client::new(local_peer_id);
    let transport = {
        let relay_tcp_quic_transport = relay_transport
            .or_transport(tcp::async_io::Transport::new(
                tcp::Config::default().port_reuse(true),
            ))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key).unwrap())
            .multiplex(yamux::Config::default());

        block_on(DnsConfig::system(relay_tcp_quic_transport))
            .unwrap()
            .map(|either_output, _| (either_output.0, StreamMuxerBox::new(either_output.1)))
            .boxed()
    };

    #[derive(NetworkBehaviour)]
    #[behaviour(to_swarm = "Event")]
    struct Behaviour {
        relay_client: relay::client::Behaviour,
        ping: ping::Behaviour,
        identify: identify::Behaviour,
        dcutr: dcutr::Behaviour,
        request_response: json::Behaviour<Secret, SecretResponse>,
    }

    #[derive(Debug)]
    #[allow(clippy::large_enum_variant)]
    enum Event {
        Ping(ping::Event),
        Identify(identify::Event),
        Relay(relay::client::Event),
        Dcutr(dcutr::Event),
        RequestResonse(request_response::Event<Secret, SecretResponse>),
    }

    impl From<ping::Event> for Event {
        fn from(e: ping::Event) -> Self {
            Event::Ping(e)
        }
    }

    impl From<identify::Event> for Event {
        fn from(e: identify::Event) -> Self {
            Event::Identify(e)
        }
    }

    impl From<relay::client::Event> for Event {
        fn from(e: relay::client::Event) -> Self {
            Event::Relay(e)
        }
    }

    impl From<dcutr::Event> for Event {
        fn from(e: dcutr::Event) -> Self {
            Event::Dcutr(e)
        }
    }

    impl From<request_response::Event<Secret, SecretResponse>> for Event {
        fn from(e: request_response::Event<Secret, SecretResponse>) -> Self {
            Event::RequestResonse(e)
        }
    }

    let behaviour = Behaviour {
        relay_client: client,
        ping: ping::Behaviour::new(ping::Config::new()),
        identify: identify::Behaviour::new(identify::Config::new(
            "/SHARE/0.0.1".to_string(),
            local_key.public(),
        )),
        dcutr: dcutr::Behaviour::new(local_peer_id),
        request_response: json::Behaviour::<Secret, SecretResponse>::new(
            [(
                StreamProtocol::new("/share-json-protocol"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        ),
    };
    let mut swarm = match ThreadPool::new() {
        Ok(tp) => SwarmBuilder::with_executor(transport, behaviour, local_peer_id, tp),
        Err(_) => SwarmBuilder::without_executor(transport, behaviour, local_peer_id),
    }
    .build();

    swarm
        .listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse().unwrap())
        .unwrap();

    // Wait to listen on all interfaces.
    block_on(async {
        let mut delay = futures_timer::Delay::new(std::time::Duration::from_secs(1)).fuse();
        loop {
            futures::select! {
                event = swarm.next() => {
                    match event.unwrap() {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {:?}", address);
                        }
                        event =>  { error!("{event:?}"); exit(1); }
                    }
                }
                _ = delay => {
                    // Likely listening on all interfaces now, thus continuing by breaking the loop.
                    break;
                }
            }
        }
    });

    // Connect to the relay server. Not for the reservation or relayed connection, but to (a) learn
    // our local public address and (b) enable a freshly started relay to learn its public address.
    swarm.dial(relay_address.clone()).unwrap();
    block_on(async {
        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;

        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { .. } => {}
                SwarmEvent::Dialing { .. } => {}
                SwarmEvent::ConnectionEstablished { .. } => {}
                SwarmEvent::Behaviour(Event::Ping(_)) => {}
                SwarmEvent::Behaviour(Event::Identify(identify::Event::Sent { .. })) => {
                    info!("Told relay its public address.");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(Event::Identify(identify::Event::Received {
                    info: identify::Info { observed_addr, .. },
                    ..
                })) => {
                    info!("Relay told us our public address: {:?}", observed_addr);
                    swarm.add_external_address(observed_addr);
                    learned_observed_addr = true;
                }
                event => panic!("{event:?}"),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }
    });

    //denotes whether to send or receive secrets
    match opts.mode {
        Mode::Send => {
            swarm
                .dial(
                    relay_address
                        .with(Protocol::P2pCircuit)
                        .with(Protocol::P2p(opts.remote_peer_id.unwrap())),
                )
                .unwrap();
        }
        Mode::Receive => {
            swarm
                .listen_on(relay_address.with(Protocol::P2pCircuit))
                .unwrap();
        }
    }

    block_on(async {
        loop {
            match swarm.next().await.unwrap() {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(Event::Relay(
                    relay::client::Event::ReservationReqAccepted { .. },
                )) => {
                    assert!(opts.mode == Mode::Receive);
                    info!("Relay accepted our reservation request.");
                }
                SwarmEvent::Behaviour(Event::Relay(event)) => {
                    info!("{:?}", event)
                }
                SwarmEvent::Behaviour(Event::Dcutr(event)) => {
                    info!("{:?}", event)
                }
                SwarmEvent::Behaviour(Event::Identify(event)) => {
                    info!("{:?}", event)
                }
                SwarmEvent::Behaviour(Event::Ping(_)) => {}
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    info!("Established connection to {:?} via {:?}", peer_id, endpoint);

                    match opts.mode {
                        Mode::Send => {
                            let request = Secret {
                                key: "key1".to_string(),
                                value: "value1".to_string(),
                            };
                            info!("Sending secret: {:#?}", request);
                            swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer_id, request);
                        }
                        Mode::Receive => {}
                    }
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    info!("Outgoing connection error to {:?}: {:?}", peer_id, error);
                }
                SwarmEvent::Behaviour(Event::RequestResonse(
                    request_response::Event::Message { peer, message },
                )) => match message {
                    request_response::Message::Request {
                        request_id: _,
                        request,
                        channel,
                    } => {
                        info!("Received secrets: {:#?} from {:?}", request, peer);
                        //TODO handle error
                        swarm
                            .behaviour_mut()
                            .request_response
                            .send_response(
                                channel,
                                SecretResponse {
                                    status: "SUCCESS".to_string(),
                                },
                            )
                            .unwrap();
                    }
                    request_response::Message::Response {
                        request_id: _,
                        response,
                    } => {
                        info!("Received {} from {:?}", response.status, peer);
                    }
                },
                _ => {}
            }
        }
    })
}

fn generate_ed25519(secret_key_seed: u8) -> identity::Keypair {
    let mut bytes = [0u8; 32];
    bytes[0] = secret_key_seed;

    identity::Keypair::ed25519_from_bytes(bytes).expect("only errors on wrong length")
}
