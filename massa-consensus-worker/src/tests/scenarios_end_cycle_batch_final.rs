use super::tools::*;
use massa_consensus_exports::{tools, ConsensusConfig};

use massa_models::ledger_models::LedgerData;
use massa_models::{Amount, BlockId, Slot};
use massa_signature::{generate_random_private_key, PrivateKey};
use massa_time::MassaTime;
use num::rational::Ratio;
use serial_test::serial;
use std::collections::HashMap;
use std::collections::{HashSet, VecDeque};
use std::str::FromStr;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::prelude::*;

#[tokio::test]
#[serial]
async fn test_end_cycle_batch_final() {
    let tracing_layer = tracing_subscriber::fmt::layer().with_filter(filter_fn(|metadata| {
        metadata.target().starts_with("massa") // ignore non-massa logs
    }));
    // build a `Subscriber` by combining layers with a `tracing_subscriber::Registry`:
    tracing_subscriber::registry()
        // add the console layer to the subscriber or default layers...
        .with(tracing_layer)
        .init();
    let staking_keys: Vec<PrivateKey> = (0..1).map(|_| generate_random_private_key()).collect();
    let init_time: MassaTime = 100.into();
    let mut cfg = ConsensusConfig {
        periods_per_cycle: 3,
        thread_count: 2,
        delta_f0: 3,
        t0: 100.into(),
        pos_miss_rate_deactivation_threshold: Ratio::new(0, 100),
        genesis_timestamp: MassaTime::now().unwrap(),
        ..ConsensusConfig::default_with_staking_keys(&staking_keys)
    };

    let (address_1, priv_1, _) = random_address_on_thread(0, cfg.thread_count).into();

    let initial_rolls_file = tools::generate_default_roll_counts_file(vec![priv_1]);
    cfg.initial_rolls_path = initial_rolls_file.path().to_path_buf();

    consensus_without_pool_test(
        cfg.clone(),
        async move |mut protocol_controller, consensus_command_sender, consensus_event_receiver| {
            let parents: Vec<BlockId> = consensus_command_sender
                .get_block_graph_status(None, None)
                .await
                .expect("could not get block graph status")
                .best_parents
                .iter()
                .map(|(b, _p)| *b)
                .collect();

            let hash_1 = create_and_test_block(
                &mut protocol_controller,
                &cfg,
                Slot::new(1, 0),
                parents.clone(),
                true,
                false,
                priv_1,
            )
            .await;

            let hash_2 = create_and_test_block(
                &mut protocol_controller,
                &cfg,
                Slot::new(1, 1),
                parents.clone(),
                true,
                false,
                priv_1,
            )
            .await;

            let hash_3 = create_and_test_block(
                &mut protocol_controller,
                &cfg,
                Slot::new(2, 0),
                parents.clone(),
                true,
                false,
                priv_1,
            )
            .await;

            let status = consensus_command_sender
                .get_block_graph_status(None, None)
                .await
                .expect("could not get block graph status");

            if hash_1 > hash_3 {
                assert_eq!(status.best_parents[0].0, hash_3);
            } else {
                assert_eq!(status.best_parents[0].0, hash_1);
            }
            assert_eq!(status.best_parents[1].0, hash_2);

            assert!(if let Some(h) = status.gi_head.get(&hash_3) {
                h.contains(&hash_1)
            } else {
                panic!("missing hash in gi_head")
            });

            assert_eq!(status.max_cliques.len(), 2);

            for clique in status.max_cliques.clone() {
                if clique.block_ids.contains(&hash_1) && clique.block_ids.contains(&hash_3) {
                    panic!("incompatible blocks in the same clique")
                }
            }

            let mut current_period = 3;
            let mut parents = vec![hash_1, hash_2];
            for _ in 0..3 {
                let hash = create_and_test_block(
                    &mut protocol_controller,
                    &cfg,
                    Slot::new(current_period, 0),
                    parents.clone(),
                    true,
                    false,
                    priv_1,
                )
                .await;
                current_period += 1;
                parents[0] = hash;
            }
            println!("period: {}\n", current_period);
            // Add 1 block in clique 2
            let hash_6_0 = create_and_test_block(
                &mut protocol_controller,
                &cfg,
                Slot::new(current_period, 0),
                vec![hash_3, parents[1]],
                true,
                false,
                priv_1,
            )
            .await;

            current_period += 1;
            //Add one in clique 1
            let hash_7_0 = create_and_test_block(
                &mut protocol_controller,
                &cfg,
                Slot::new(current_period, 0),
                parents.clone(),
                true,
                false,
                priv_1,
            )
            .await;
            parents[0] = hash_7_0;
            current_period += 1;

            //Add block clique 2 (test)
            let hash_8_0 = create_and_test_block(
                &mut protocol_controller,
                &cfg,
                Slot::new(current_period, 0),
                vec![hash_6_0, parents[1]],
                true,
                false,
                priv_1,
            )
            .await;

            current_period += 1;
            //Add one in clique 1

            let status = consensus_command_sender
                .get_block_graph_status(None, None)
                .await
                .expect("could not get block graph status");
            for (i, clique) in status.max_cliques.iter().enumerate() {
                println!("clique1: {}, clique content: {:#?}", i, clique);
            }
            println!("{}", status.max_cliques.len());
            for active in status.active_blocks {
                println!(
                    "blockid: {:#?}, final? {}, period: {:#?}, thread: {:#?}\nParents:",
                    active.0,
                    active.1.is_final,
                    active.1.header.content.slot.period,
                    active.1.header.content.slot.thread
                );
                for parent in active.1.header.content.parents {
                    println!("    {:#?}", parent);
                }
                println!("Children:");
                for child in active.1.children {
                    println!("    {:#?}", child);
                }
                println!("=======");
            }

            //end test

            let rs_a2_r1 = create_roll_sell(priv_1, 1, 90, 0);
            let (_, block1_err1, _) = create_block_with_operations(
                &cfg,
                Slot::new(current_period, 0),
                &parents,
                priv_1,
                vec![rs_a2_r1],
            );
            propagate_block(&mut protocol_controller, block1_err1, false, 250).await;
            println!("{:#?}", current_period);

            let status = consensus_command_sender
                .get_block_graph_status(None, None)
                .await
                .expect("could not get block graph status");
            for (i, clique) in status.max_cliques.iter().enumerate() {
                println!("clique2: {}, clique content: {:#?}", i, clique);
            }
            println!("{}", status.max_cliques.len());
            for active in status.active_blocks {
                println!(
                    "blockid: {:#?}, final? {}, period: {:#?}, thread: {:#?}\nParents:",
                    active.0,
                    active.1.is_final,
                    active.1.header.content.slot.period,
                    active.1.header.content.slot.thread
                );
                for parent in active.1.header.content.parents {
                    println!("    {:#?}", parent);
                }
                println!("Children:");
                for child in active.1.children {
                    println!("    {:#?}", child);
                }
                println!("=======");
            }
            /*let parents: Vec<BlockId> = consensus_command_sender
                .get_block_graph_status(None, None)
                .await
                .expect("could not get block graph status")
                .best_parents
                .iter()
                .map(|(b, _p)| *b)
                .collect();
            let mut clique_1: Vec<BlockId> = Vec::new();
            clique_1.push(parents[0]);
            let mut clique_2: Vec<BlockId> = Vec::new();
            clique_2.push(parents[0]);
            for period in 1..=7 {
                println!("{:#?}", period);
                clique_1.push(
                    create_and_test_block(
                        &mut protocol_controller,
                        &cfg,
                        Slot::new(period, 0),
                        vec! [*clique_1.last().unwrap()],
                        true,
                        false,
                        priv_1,
                    ).await
                );
            }

            for period in 1..=4 {
                if period <= 4 {
                    clique_2.push(
                        create_and_test_block(
                            &mut protocol_controller,
                            &cfg,
                            Slot::new(period, 0),
                            vec! [*clique_2.last().unwrap()],
                            true,
                            false,
                            priv_1,
                        ).await
                    );
                }
            }*/
            (
                protocol_controller,
                consensus_command_sender,
                consensus_event_receiver,
            )
        },
    )
    .await;
}
