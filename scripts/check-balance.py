#!/usr/bin/env python3
"""
Polymarket 测试钱包余额查询脚本

使用方法:
    python3 scripts/check-balance.py [钱包地址]

示例:
    python3 scripts/check-balance.py
    python3 scripts/check-balance.py 0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6
"""

import sys
from decimal import Decimal

try:
    from web3 import Web3
except ImportError:
    print("❌ 错误：缺少 web3 库")
    print("安装命令：pip3 install web3")
    sys.exit(1)

# 默认测试钱包
DEFAULT_WALLET = "0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6"

# Polygon 主网 RPC 端点列表（按优先级排序）
RPC_ENDPOINTS = [
    # 快速响应
    "https://rpc.sentio.xyz/matic",
    "https://rpc.owlracle.info/poly/70d38ce1826c4a60bb2a8e05a6c8b20f",
    "https://polygon-public.nodies.app",
    "https://gateway.tenderly.co/public/polygon",
    "https://1rpc.io/matic",
    
    # 稳定服务
    "https://polygon-bor-rpc.publicnode.com",
    "https://polygon-bor.publicnode.com",
    "https://go.getblock.io/02667b699f05444ab2c64f9bff28f027",
    "https://api.zan.top/polygon-mainnet",
    "https://poly.api.pocket.network",
    
    # 备用
    "https://rpc.ankr.com/polygon",
    "https://api-polygon-mainnet-full.n.dwellir.com/2ccf18bf-2916-4198-8856-42172854353c",
]

# USDC 合约地址 (Polygon)
USDC_CONTRACT = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"

# USDC ABI (minimal)
USDC_ABI = [{
    "constant": True,
    "inputs": [{"name": "_owner", "type": "address"}],
    "name": "balanceOf",
    "outputs": [{"name": "balance", "type": "uint256"}],
    "type": "function"
}]

# MATIC 价格 (USD)
MATIC_PRICE_USD = 0.45


def connect_to_polygon(verbose=True):
    """连接到 Polygon 主网"""
    if verbose:
        print("正在连接 Polygon 主网...")
    
    for rpc in RPC_ENDPOINTS:
        try:
            w3 = Web3(Web3.HTTPProvider(rpc))
            if w3.is_connected():
                if verbose:
                    print(f"✅ 已连接到：{rpc[:60]}...\n")
                return w3, rpc
        except Exception:
            continue
    
    if verbose:
        print("❌ 无法连接到任何 RPC 端点")
    return None, None


def get_matic_balance(w3, address):
    """获取 MATIC 余额"""
    balance_wei = w3.eth.get_balance(address)
    return float(w3.from_wei(balance_wei, "ether"))


def get_usdc_balance(w3, address):
    """获取 USDC 余额"""
    try:
        usdc = w3.eth.contract(address=USDC_CONTRACT, abi=USDC_ABI)
        balance = usdc.functions.balanceOf(address).call()
        return balance / 1_000_000  # USDC 有 6 位小数
    except Exception as e:
        return None


def print_balance_report(wallet, matic, usdc):
    """打印余额报告"""
    print("=" * 70)
    print("💰 Polymarket 测试钱包余额报告")
    print("=" * 70)
    print(f"\n钱包地址：{wallet}")
    print(f"网络：Polygon 主网")
    print()
    
    # MATIC
    print(f"MATIC 余额：{matic:.4f} MATIC")
    print(f"  美元价值：≈ ${matic * MATIC_PRICE_USD:.2f} (按 ${MATIC_PRICE_USD}/MATIC)")
    
    # Gas 估算
    gas_cost = 0.01  # 单次交易约 0.01 MATIC
    tx_count = int(matic / gas_cost) if gas_cost > 0 else 0
    print(f"  可支持交易：≈ {tx_count} 笔 (按 {gas_cost} MATIC/笔)")
    print()
    
    # USDC
    if usdc is not None:
        print(f"USDC 余额：{usdc:.2f} USDC")
        print(f"  美元价值：≈ ${usdc:.2f}")
        
        # 测试订单估算 (最小订单 0.1 USDC)
        min_order = 0.1
        order_count = int(usdc / min_order) if min_order > 0 else 0
        print(f"  可支持订单：≈ {order_count} 笔 (按 {min_order} USDC/笔)")
    else:
        print("USDC 余额：查询失败")
    print()
    
    # 评估
    print("=" * 70)
    print("📊 资金评估")
    print("=" * 70)
    
    # MATIC 评估
    if matic < 0.5:
        print("⚠️  MATIC: 余额不足，建议充值 1+ MATIC")
        matic_ok = False
    else:
        print("✅ MATIC: 余额充足，可用于测试")
        matic_ok = True
    
    # USDC 评估
    if usdc is None:
        print("⚠️  USDC: 查询失败")
        usdc_ok = False
    elif usdc < 0.5:
        print(f"⚠️  USDC: 余额不足 ({usdc:.2f}), 建议充值 5+ USDC")
        usdc_ok = False
    elif usdc < 2:
        print(f"⚠️  USDC: 余额偏低 ({usdc:.2f}), 建议充值 10+ USDC")
        usdc_ok = False
    else:
        print("✅ USDC: 余额充足，可用于测试")
        usdc_ok = True
    
    print()
    print("=" * 70)
    
    if matic_ok and usdc_ok:
        print("✅ 资金充足，可以开始主网测试！")
    elif matic_ok:
        print("⚠️  MATIC 充足，但 USDC 不足 - 可进行 Gas 测试，无法实际交易")
    else:
        print("⚠️  建议先充值再开始测试")
    
    print("=" * 70)
    
    # 充值信息
    if not matic_ok or not usdc_ok:
        print()
        print("💡 充值信息:")
        print(f"  钱包地址：{wallet}")
        print(f"  网络：Polygon 主网")
        print(f"  建议充值：1+ MATIC, 5+ USDC")
        print()


def list_rpc_endpoints():
    """列出所有可用的 RPC 端点"""
    print("=" * 70)
    print("📡 Polygon 主网 RPC 端点列表")
    print("=" * 70)
    print()
    
    for i, rpc in enumerate(RPC_ENDPOINTS, 1):
        print(f"{i:2}. {rpc}")
    
    print()
    print("=" * 70)
    print(f"总计：{len(RPC_ENDPOINTS)} 个端点")
    print()


def main():
    # 特殊命令：列出 RPC 端点
    if len(sys.argv) > 1 and sys.argv[1] in ["--list", "-l"]:
        list_rpc_endpoints()
        return
    
    # 获取钱包地址
    if len(sys.argv) > 1:
        wallet = sys.argv[1]
    else:
        wallet = DEFAULT_WALLET
    
    # 验证地址格式
    if not Web3.is_address(wallet):
        print(f"❌ 错误：无效的钱包地址 '{wallet}'")
        sys.exit(1)
    
    wallet = Web3.to_checksum_address(wallet)
    
    print(f"正在查询钱包余额...\n")
    
    # 连接到 Polygon
    w3, rpc = connect_to_polygon()
    if not w3:
        print("❌ 错误：无法连接到 Polygon 主网")
        print("请检查网络连接或尝试稍后重试")
        print("\n提示：运行 'python3 scripts/check-balance.py --list' 查看所有 RPC 端点")
        sys.exit(1)
    
    # 查询余额
    try:
        matic = get_matic_balance(w3, wallet)
        usdc = get_usdc_balance(w3, wallet)
        print_balance_report(wallet, matic, usdc)
    except Exception as e:
        print(f"❌ 查询失败：{e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
