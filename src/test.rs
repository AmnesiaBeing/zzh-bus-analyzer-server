use std::collections::HashMap;
use std::time::Instant;
use rand::Rng;

// 定义树节点结构体，使用 HashMap 作为子节点容器
struct TreeNodeHashMap {
    value: i32,
    children: HashMap<String, TreeNodeHashMap>,
}

impl TreeNodeHashMap {
    fn new(value: i32) -> Self {
        TreeNodeHashMap {
            value,
            children: HashMap::new(),
        }
    }

    fn add_child(&mut self, key: String, child: TreeNodeHashMap) {
        self.children.insert(key, child);
    }

    fn get_child(&self, key: &str) -> Option<&TreeNodeHashMap> {
        self.children.get(key)
    }
}

// 定义树节点结构体，使用 Vec 作为子节点容器
struct TreeNodeVec {
    value: i32,
    children: Vec<(String, TreeNodeVec)>,
}

impl TreeNodeVec {
    fn new(value: i32) -> Self {
        TreeNodeVec {
            value,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, key: String, child: TreeNodeVec) {
        self.children.push((key, child));
    }

    fn get_child(&self, key: &str) -> Option<&TreeNodeVec> {
        self.children.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

// 生成随机字符串
fn generate_random_string(len: usize) -> String {
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
    let mut rng = rand::thread_rng();
    (0..len).map(|_| chars[rng.gen_range(0..chars.len())]).collect()
}

// 构建随机键列表
fn generate_keys(levels: usize, max_children: usize) -> Vec<Vec<String>> {
    let mut keys = Vec::new();
    for level in 0..levels {
        let mut level_keys = Vec::new();
        for _ in 0..max_children.pow(level as u32) {
            level_keys.push(generate_random_string(10));
        }
        keys.push(level_keys);
    }
    keys
}

// 使用相同的键列表构建随机树
fn build_tree_hashmap(keys: &Vec<Vec<String>>, levels: usize, current_level: usize) -> TreeNodeHashMap {
    let mut node = TreeNodeHashMap::new(current_level as i32);
    if current_level < levels {
        for key in &keys[current_level] {
            let child = build_tree_hashmap(keys, levels, current_level + 1);
            node.add_child(key.clone(), child);
        }
    }
    node
}

fn build_tree_vec(keys: &Vec<Vec<String>>, levels: usize, current_level: usize) -> TreeNodeVec {
    let mut node = TreeNodeVec::new(current_level as i32);
    if current_level < levels {
        for key in &keys[current_level] {
            let child = build_tree_vec(keys, levels, current_level + 1);
            node.add_child(key.clone(), child);
        }
    }
    node
}

#[test]
fn main() {
    let levels = 3; // 树的层数（2至4层，可以调整）
    let max_children = 100; // 每个节点的最大子节点数
    let keys = generate_keys(levels, max_children); // 生成随机键列表
    let search_key = keys[0][0].clone(); // 从生成的键中选择一个作为搜索的key

    // 构建并测试使用 HashMap 的树
    let root_hashmap = build_tree_hashmap(&keys, levels, 0);
    let start = Instant::now();
    for _ in 0..10000 {
        let mut current_node = &root_hashmap;
        for level in 0..levels {
            if let Some(child) = current_node.get_child(&keys[level][0]) {
                current_node = child;
            } else {
                break;
            }
        }
    }
    let duration = start.elapsed();
    println!("HashMap lookup duration: {:?}", duration);

    // 构建并测试使用 Vec 的树
    let root_vec = build_tree_vec(&keys, levels, 0);
    let start = Instant::now();
    for _ in 0..10000 {
        let mut current_node = &root_vec;
        for level in 0..levels {
            if let Some(child) = current_node.get_child(&keys[level][0]) {
                current_node = child;
            } else {
                break;
            }
        }
    }
    let duration = start.elapsed();
    println!("Vec lookup duration: {:?}", duration);
}
