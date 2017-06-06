#[derive(Serialize, Deserialize, Debug)]
pub struct ServerBlock {
    pub port: u32,
    pub source: String,
    pub root: String,
    pub base: String,
    pub host: String
}

#[derive(Debug)]
pub struct AccumulatedServerBlock {
    pub port: u32,
    pub blocks: Vec<ServerBlock>
}

pub fn accumlated_server_blocks(blocks: Vec<ServerBlock>) -> Vec<AccumulatedServerBlock> {
    let mut ports: Vec<u32> = blocks
        .iter()
        .map(|b| b.port)
        .collect();

    ports.sort();
    ports.dedup();

    let mut base_accumulator: Vec<AccumulatedServerBlock> = ports
        .iter()
        .map(|p| AccumulatedServerBlock { port: *p, blocks: Vec::new() })
        .collect();

    for block in blocks {
        let mut parent_ref = base_accumulator.iter_mut().find(|asb| asb.port == block.port).unwrap();
        parent_ref.blocks.push(block)
    }

    base_accumulator
}

#[cfg(test)]
mod tests {
    use super::accumlated_server_blocks;
    use super::ServerBlock;

    #[test]
    fn accumulated_server_block_correctly_aggregates() {
        let first_block = ServerBlock { port: 100, source: "xa".to_string(), root: "ya".to_string(), base: "za".to_string(), host: "localhost".to_string() };
        let second_block = ServerBlock { port: 100, source: "xb".to_string(), root: "yb".to_string(), base: "zb".to_string(), host: "localhost".to_string() };
        let third_block = ServerBlock { port: 200, source: "xc".to_string(), root: "yc".to_string(), base: "zc".to_string(), host: "localhost".to_string() };
        let mut test_vec = Vec::new();
        test_vec.push(first_block);
        test_vec.push(second_block);
        test_vec.push(third_block);

        let result = accumlated_server_blocks(test_vec);
        assert_eq!(result.len(), 2);
        println!("result: {:?}", result);
        assert_eq!(result[0].blocks[0].source, "xa");
        assert_eq!(result[0].blocks[1].source, "xb");
        assert_eq!(result[1].blocks[0].source, "xc");
    }
}
