use std::time::Duration;

use anyhow::Result;
use libp2p::{
    core::upgrade::Version,
    dns,
    futures::StreamExt,
    identify,
    identity::{self, Keypair},
    noise, ping,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};

use crate::cli::{Command, IdentifyArgs, PingArgs};

pub enum Operation {
    Ping(PingArgs),
    Identify(IdentifyArgs),
}

impl TryFrom<Command> for Operation {
    type Error = Command;

    fn try_from(value: Command) -> std::result::Result<Self, Self::Error> {
        match value {
            Command::P2pPing(args) => Ok(Operation::Ping(args)),
            Command::P2pIdentify(args) => Ok(Operation::Identify(args)),
            _ => Err(value),
        }
    }
}

pub async fn run(op: Operation) -> Result<()> {
    match op {
        Operation::Ping(args) => {
            let local_key = identity::Keypair::generate_ed25519();
            let mut swarm = p2p_swarm(
                local_key,
                ping::Behaviour::new(
                    ping::Config::new()
                        .with_interval(Duration::from_secs(args.interval as u64))
                        .with_timeout(Duration::from_secs(args.timeout as u64)),
                ),
            )
            .await?;
            let remote: Multiaddr = args.peer_addr.parse()?;
            swarm.dial(remote)?;

            let mut count = 0;
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        println!("failed to connect to {peer_id:?}: {error}");
                        break;
                    }
                    SwarmEvent::Behaviour(ping::Event { peer, result, .. }) => {
                        match result {
                            Ok(duration) => println!("response from {peer} in {duration:?}"),
                            Err(err) => {
                                println!("ping failed {err}");
                                break;
                            }
                        };
                        count += 1;
                        if count >= args.count {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
        Operation::Identify(args) => {
            let local_key = identity::Keypair::generate_ed25519();
            let public_key = local_key.public();
            let mut swarm = p2p_swarm(
                local_key,
                identify::Behaviour::new(identify::Config::new(
                    "/ipfs/id/1.0.0".to_string(),
                    public_key,
                )),
            )
            .await?;
            let remote: Multiaddr = args.peer_addr.parse()?;
            swarm.dial(remote)?;

            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                        println!("failed to connect to {peer_id:?}: {error}");
                        break;
                    }
                    SwarmEvent::Behaviour(event) => match event {
                        identify::Event::Received { peer_id, info } => {
                            let public_key_type = info.public_key.key_type();
                            let protocol_version = info.protocol_version;
                            let agent_version = info.agent_version;
                            let listen_addrs = info
                                .listen_addrs
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<String>>()
                                .join("\n\t");
                            let protocols = info
                                .protocols
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<String>>()
                                .join("\n\t");
                            let observed_address = info.observed_addr;
                            println!(
                                "Peer: {peer_id}
Public Key Type: {public_key_type}
Protocol Version: {protocol_version}
Agent Version: {agent_version}
Observed Address: {observed_address}
Listen Addresses:
\t{listen_addrs}
Protocols:
\t{protocols}"
                            );
                            break;
                        }
                        identify::Event::Error { error, .. } => {
                            println!("Error getting peer identity: {error}")
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
async fn p2p_swarm<B>(local_key: Keypair, behaviour: B) -> Result<Swarm<B>>
where
    B: NetworkBehaviour,
{
    let local_peer_id = PeerId::from(local_key.public());

    let transport = tcp::tokio::Transport::default()
        .upgrade(Version::V1Lazy)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let dns_cfg = dns::ResolverConfig::cloudflare();
    let dns_opts = dns::ResolverOpts::default();
    let transport = dns::TokioDnsConfig::custom(transport, dns_cfg, dns_opts)
        .unwrap()
        .boxed();

    let mut swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id)
        .idle_connection_timeout(Duration::from_secs(60)) // For illustrative purposes, keep idle connections alive for a minute so we can observe a few pings.
        .build();

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    Ok(swarm)
}
