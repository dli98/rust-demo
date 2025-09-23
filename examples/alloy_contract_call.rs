use alloy::{
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    sol,
};
use eyre::Result;

// 定义一个简单的 ERC20 合约接口
sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract IERC20 {
        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        
        event Transfer(address indexed from, address indexed to, uint256 value);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Alloy 合约调用示例");
    
    // 连接到以太坊主网 (使用公共 RPC)
    let rpc_url = "https://eth.llamarpc.com".parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);
    
    // 尝试多个知名的 ERC20 合约地址
    let contracts_to_try = vec![
        ("USDT", address!("dAC17F958D2ee523a2206206994597C13D831ec7")),
        ("USDC", address!("A0b86a33E6441b8C4505B4afDcA7aBB2B6e1FD79")),
        ("WETH", address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")),
    ];
    
    let mut successful_contract = None;
    
    // 尝试找到一个可用的合约
    for (token_name, contract_address) in contracts_to_try {
        println!("🔍 尝试连接 {} 合约: {}", token_name, contract_address);
        let contract = IERC20::new(contract_address, &provider);
        
        // 测试合约是否可用
        match contract.name().call().await {
            Ok(_name) => {
                println!("✅ 成功连接到 {} 合约!", token_name);
                // 这里发生了所有权转移：
                // 1. Some((token_name, contract, contract_address)) 创建临时值
                // 2. 临时值被移动到 successful_contract 中
                // 3. Some 构造函数的生命周期结束，但数据所有权已转移
                // 4. token_name 是 &str，会被复制；contract 和 contract_address 被移动
                successful_contract = Some((token_name, contract, contract_address));
                break;
            },
            Err(e) => {
                println!("❌ {} 合约连接失败: {}", token_name, e);
                continue;
            }
        }
    }
    
    // 从 Option 中提取数据，获得所有权
    // 这里 successful_contract 被消费（moved），数据所有权转移到新变量
    let (token_name, contract, contract_address) = match successful_contract {
        Some(contract_info) => contract_info, // contract_info 是一个元组，被解构并移动
        None => {
            println!("❌ 所有合约都连接失败，程序退出");
            return Ok(());
        }
    };
    
    println!("\n📋 正在查询 {} 合约信息...", token_name);
    
    // 调用合约的只读方法
    match contract.name().call().await {
        Ok(name) => println!("代币名称: {}", name._0),
        Err(e) => println!("获取名称失败: {}", e),
    }

    match contract.symbol().call().await {
        Ok(symbol) => println!("代币符号: {}", symbol._0),
        Err(e) => println!("获取符号失败: {}", e),
    }
    
    match contract.decimals().call().await {
        Ok(decimals) => println!("小数位数: {}", decimals._0),
        Err(e) => println!("获取小数位数失败: {}", e),
    }
    
    let decimals = match contract.decimals().call().await {
        Ok(decimals) => decimals._0,
        Err(_) => 18, // 默认使用 18 位小数
    };
    
    match contract.totalSupply().call().await {
        Ok(total_supply) => {
            let supply_formatted = format_token_amount(total_supply._0, decimals);
            println!("总供应量: {} {}", supply_formatted, token_name);
        },
        Err(e) => println!("获取总供应量失败: {}", e),
    }
    
    // 查询特定地址的余额 (Vitalik 的地址)
    let vitalik_address = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    match contract.balanceOf(vitalik_address).call().await {
        Ok(balance) => {
            let balance_formatted = format_token_amount(balance._0, decimals);
            println!("Vitalik 的 {} 余额: {} {}", token_name, balance_formatted, token_name);
        },
        Err(e) => println!("获取余额失败: {}", e),
    }
    
    // 获取网络级别的区块链信息（与具体合约无关）
    println!("\n🔗 以太坊网络信息:");
    match provider.get_block_number().await {
        Ok(block_number) => println!("网络当前区块高度: {} (整个以太坊网络的最新区块)", block_number),
        Err(e) => println!("获取网络区块高度失败: {}", e),
    }
    
    // 获取网络 Chain ID
    match provider.get_chain_id().await {
        Ok(chain_id) => {
            let network_name = match chain_id {
                1 => "以太坊主网",
                5 => "Goerli 测试网",
                11155111 => "Sepolia 测试网",
                137 => "Polygon 主网",
                _ => "未知网络",
            };
            println!("网络 Chain ID: {} ({})", chain_id, network_name);
        },
        Err(e) => println!("获取 Chain ID 失败: {}", e),
    }
    
    // 获取合约相关的额外信息
    println!("\n📊 合约相关信息:");
    println!("合约地址: {}", contract_address);
    
    // 获取合约地址的 ETH 余额
    match provider.get_balance(contract_address).await {
        Ok(balance) => {
            let eth_balance = format_token_amount(balance, 18);
            println!("合约地址的 ETH 余额: {} ETH", eth_balance);
        },
        Err(e) => println!("获取合约 ETH 余额失败: {}", e),
    }
    
    // 尝试获取合约的字节码（某些 RPC 提供商可能不支持此方法）
    println!("合约地址验证: {} ✅ (已通过合约调用验证)", contract_address);
    
    println!("\n✅ 合约调用示例完成!");
    
    Ok(())
}

// 辅助函数：格式化代币数量
fn format_token_amount(amount: U256, decimals: u8) -> String {
    let divisor = U256::from(10).pow(U256::from(decimals));
    let whole = amount / divisor;
    let remainder = amount % divisor;
    
    if remainder.is_zero() {
        whole.to_string()
    } else {
        let remainder_str = format!("{:0width$}", remainder, width = decimals as usize);
        let trimmed = remainder_str.trim_end_matches('0');
        if trimmed.is_empty() {
            whole.to_string()
        } else {
            format!("{}.{}", whole, trimmed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_token_amount() {
        // 测试格式化函数
        assert_eq!(format_token_amount(U256::from(1000000), 6), "1");
        assert_eq!(format_token_amount(U256::from(1500000), 6), "1.5");
        assert_eq!(format_token_amount(U256::from(1234567), 6), "1.234567");
    }
}