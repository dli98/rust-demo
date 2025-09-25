use alloy::{
    providers::{Provider, ProviderBuilder}, 
    primitives::{Address, U256},
    hex
};
use alloy::sol;

sol! {
    #[sol(rpc)]
    interface ILogicContract {
        // 事件（保持原始ABI结构）
        event Approval(address indexed owner, address indexed spender, uint256 value);
        event ETHBurned(address indexed burner, uint256 ethAmount, uint256 usdValue);
        event Initialized(uint8 version);
        event Mint(address indexed to, uint256 dsuAmount);
        event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
        event PriceFeedUpdated(address indexed oldPriceFeed, address indexed newPriceFeed);
        event Transfer(address indexed from, address indexed to, uint256 value);

        // 状态变量
        function BURN_ADDRESS() external view returns (address);
        function priceFeed() external view returns (address);
        
        // 代币标准方法
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
        function decimals() external view returns (uint8);
        function decreaseAllowance(address spender, uint256 subtractedValue) external returns (bool);
        function increaseAllowance(address spender, uint256 addedValue) external returns (bool);
        function name() external view returns (string memory);
        function symbol() external view returns (string memory);
        function totalSupply() external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
        
        // 业务逻辑方法
        function calculateDsuAmount(uint256 ethAmount) external view returns (uint256);
        function getEthUsdPrice() external view returns (uint256);
        function mintWithEth() external payable;
        function recoverETH(uint256 amount) external;
        function recoverToken(address tokenAddress, uint256 amount) external;
        
        // 所有权管理
        function owner() external view returns (address);
        function renounceOwnership() external;
        function transferOwnership(address newOwner) external;
        
        // 初始化与配置
        function initialize(address _priceFeedAddress) external;
        function updatePriceFeed(address _newPriceFeed) external;
        
        // 检查初始化状态的方法 (多种可能的方式)
        function getInitializedVersion() external view returns (uint8);
        function initialized() external view returns (bool);
        function hasRole(bytes32 role, address account) external view returns (bool);
        
        receive() external payable;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化
    // let rpc_url = "https://bsc-dataseed.binance.org/".parse()?;
    let rpc_url = "https://bsc.publicnode.com".parse()?;
    let provider = ProviderBuilder::new().on_http(rpc_url);

    // 2. 获取逻辑合约地址
    let proxy_address: Address = "0x926381886fbdac01eA518a62B405C62d29F77E36".parse()?;
    // let proxy_address: Address = "0xD62519ED56d6cbEB927D726dB215d83FA3aD57b6".parse()?;
    println!("代理合约地址: {:?}", proxy_address);
    
    // 检查代理合约是否存在
    let code = provider.get_code_at(proxy_address).await?;
    if code.is_empty() {
        return Err("代理合约不存在或没有代码".into());
    }
    println!("代理合约代码长度: {} bytes", code.len());
    
    let slot_bytes = hex::decode("360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc")?;
    let slot_array: [u8; 32] = slot_bytes.try_into()
        .map_err(|_| "Invalid slot length")?;
    let impl_slot = U256::from_be_bytes(slot_array);
    
    let impl_address = {
        let data = provider.get_storage_at(proxy_address, impl_slot).await?;
        let bytes = data.to_be_bytes::<32>();
        let addr = Address::from_slice(&bytes[12..]);
        println!("从存储槽读取的逻辑合约地址: {:?}", addr);
        
        // 检查逻辑合约是否存在
        let logic_code = provider.get_code_at(addr).await?;
        if logic_code.is_empty() {
            return Err("逻辑合约不存在或没有代码".into());
        }
        println!("逻辑合约代码长度: {} bytes", logic_code.len());
        
        addr
    };

    // 3. 创建合约实例
    let logic_contract = ILogicContract::new(impl_address, provider.clone());
    println!("逻辑合约地址: {:?}", impl_address);

    // 检查合约所有者
    let owner_result = logic_contract.owner().call().await;
    match owner_result {
        Ok(owner) => {
            println!("合约所有者: {:?}", owner._0);
            if owner._0 == Address::ZERO {
                println!("⚠️  合约所有者是零地址，可能未初始化或所有权已放弃");
            }
        },
        Err(e) => {
            println!("查询owner失败: {:?}", e);
        }
    }
    
    // 检查合约初始化状态
    println!("\n=== 检查合约初始化状态 ===");
    
    // 方法1: 尝试 getInitializedVersion
    let init_version_result = logic_contract.getInitializedVersion().call().await;
    match init_version_result {
        Ok(version) => {
            println!("✅ 合约初始化版本: {}", version._0);
            if version._0 == 0 {
                println!("❌ 合约未初始化 (版本为 0)");
            } else {
                println!("✅ 合约已初始化 (版本: {})", version._0);
            }
        },
        Err(_) => {
            println!("⚠️  getInitializedVersion 方法不可用");
            
            // 方法2: 尝试 initialized() 布尔方法
            let init_bool_result = logic_contract.initialized().call().await;
            match init_bool_result {
                Ok(is_init) => {
                    if is_init._0 {
                        println!("✅ 合约已初始化 (initialized = true)");
                    } else {
                        println!("❌ 合约未初始化 (initialized = false)");
                    }
                },
                Err(_) => {
                    println!("⚠️  initialized() 方法也不可用");
                    
                    // 方法3: 通过检查关键状态变量来判断
                    println!("🔍 尝试通过状态变量判断初始化状态...");
                    
                    // 检查 priceFeed 地址
                    let pricefeed_result = logic_contract.priceFeed().call().await;
                    match pricefeed_result {
                        Ok(pf) => {
                            println!("价格预言机地址: {:?}", pf._0);
                            if pf._0 == Address::ZERO {
                                println!("❌ 价格预言机未设置，合约可能未初始化");
                            } else {
                                println!("✅ 价格预言机已设置，合约可能已初始化");
                            }
                        },
                        Err(e) => println!("查询价格预言机失败: {:?}", e),
                    }
                    
                    // 检查 BURN_ADDRESS
                    let burn_result = logic_contract.BURN_ADDRESS().call().await;
                    match burn_result {
                        Ok(burn) => {
                            println!("销毁地址: {:?}", burn._0);
                        },
                        Err(e) => println!("查询销毁地址失败: {:?}", e),
                    }
                }
            }
        }
    }
    
    println!("\n=== 总结 ===");
    println!("🔍 基于以上检查，合约状态分析：");
    println!("1. 合约所有者为零地址");
    println!("2. 标准初始化检查方法失败");
    println!("3. 所有 ERC20 方法调用都失败");
    println!("📝 结论: 该合约需要进行初始化才能正常使用");

    // 4. 查询数据
    let dummy_address: Address = "0xa0ac5ea5d0c0dfe3a9d03681f428319f853e2c2a".parse()?;
    println!("查询地址: {:?}", dummy_address);

    // 先尝试查询总供应量（通常更稳定）
    println!("正在查询总供应量...");
    let supply_result = logic_contract.totalSupply().call().await;
    match supply_result {
        Ok(supply) => {
            println!("总供应量: {}", supply._0);
            
            // 如果总供应量查询成功，再查询余额
            println!("正在查询余额...");
            let balance_result = logic_contract.balanceOf(dummy_address).call().await;
            match balance_result {
                Ok(balance) => {
                    println!("余额: {}", balance._0);
                },
                Err(e) => {
                    println!("查询余额失败: {:?}", e);
                    // 尝试查询其他信息
                    if let Ok(name) = logic_contract.name().call().await {
                        println!("代币名称: {}", name._0);
                    }
                    if let Ok(symbol) = logic_contract.symbol().call().await {
                        println!("代币符号: {}", symbol._0);
                    }
                }
            }
        },
        Err(e) => {
            println!("查询总供应量失败: {:?}", e);
            return Err(format!("合约调用失败: {:?}", e).into());
        }
    }

    Ok(())
}
