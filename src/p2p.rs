use std::time::Duration;

use anyhow::Result;
use libp2p::{
    futures::{pin_mut, StreamExt},
    identify,
    identity::{self, Keypair},
    noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, tls, yamux, Multiaddr, Swarm, SwarmBuilder,
};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};

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

pub async fn run(op: Operation, _stdin: impl AsyncRead, stdout: impl AsyncWrite) -> Result<()> {
    pin_mut!(stdout);
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
                        stdout
                            .write_all(
                                format!("failed to connect to {peer_id:?}: {error}\n").as_bytes(),
                            )
                            .await?;
                        break;
                    }
                    SwarmEvent::Behaviour(ping::Event { peer, result, .. }) => {
                        match result {
                            Ok(duration) => {
                                stdout
                                    .write_all(
                                        format!("response from {peer} in {duration:?}\n")
                                            .as_bytes(),
                                    )
                                    .await?
                            }
                            Err(err) => {
                                stdout
                                    .write_all(format!("ping failed {err}\n").as_bytes())
                                    .await?;
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
                        stdout
                            .write_all(
                                format!("failed to connect to {peer_id:?}: {error}\n").as_bytes(),
                            )
                            .await?;
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
                            let mut protocols = info
                                .protocols
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<String>>();
                            protocols.sort();
                            let protocols = protocols.join("\n\t");
                            let observed_address = info.observed_addr;
                            stdout
                                .write_all(
                                    format!(
                                        "Peer: {peer_id}
Public Key Type: {public_key_type}
Protocol Version: {protocol_version}
Agent Version: {agent_version}
Observed Address: {observed_address}
Listen Addresses:
\t{listen_addrs}
Protocols:
\t{protocols}
"
                                    )
                                    .as_bytes(),
                                )
                                .await?;
                            break;
                        }
                        identify::Event::Error { error, .. } => {
                            stdout
                                .write_all(
                                    format!("Error getting peer identity: {error}\n").as_bytes(),
                                )
                                .await?
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
    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_dns()?
        .with_websocket(
            (tls::Config::new, noise::Config::new),
            yamux::Config::default,
        )
        .await?
        .with_behaviour(|_| behaviour)?
        .with_swarm_config(|config| config.with_idle_connection_timeout(Duration::from_secs(30)))
        .build();

    // Tell the swarm to listen on all interfaces and a random, OS-assigned
    // port.
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    Ok(swarm)
}
