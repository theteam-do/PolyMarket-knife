#!/usr/bin/env python3
"""
测试 Polymarket API 密钥是否有效
"""

import sys

try:
    from py_clob_client.client import ClobClient
    from py_clob_client.config import ClobApiParams
except ImportError:
    print("❌ 请安装依赖：pip3 install py-clob-client")
    sys.exit(1)

# API 凭证
API_KEY = "019cb2c3-ac6d-72ba-8cc0-96e7f9e5cfab"
API_SECRET = "mh9swZPflIqnDbitI620c-nOE5o2NQ89MjusqKCysNk="
PASSPHRASE = "528546e12e12fa40f8201a363f6056df960aef49415f3768ab14418c19317905"
PRIVATE_KEY = "0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7"
CHAIN_ID = 137  # Polygon 主网
CLOB_HOST = "https://clob.polymarket.com"


def main():
    print("🔑 测试 Polymarket API 密钥")
    print("=" * 50)

    try:
        # 创建客户端
        print("📡 创建 CLOB 客户端...")
        client = ClobClient(
            host=CLOB_HOST,
            chain_id=CHAIN_ID,
            key=PRIVATE_KEY,
            creds=ClobApiParams(
                api_key=API_KEY,
                api_secret=API_SECRET,
                passphrase=PASSPHRASE,
            ),
        )
        print("✅ 客户端创建成功")

        # 测试获取余额
        print("\n💰 测试获取余额...")
        balances = client.get_balances()
        print(f"✅ 余额查询成功:")
        print(f"   USDC: {balances.get('usdc', 'N/A')}")
        print(f"   MATIC: {balances.get('matic', 'N/A')}")

        # 测试获取订单簿
        print("\n📊 测试获取订单簿...")
        token_id = "8501497159083948713316135768103773293754490207922884688769443031624417212426"
        book = client.get_orderbook(token_id)
        print(f"✅ 订单簿获取成功:")
        print(f"   Token: {book.get('token_id', 'N/A')}")
        print(f"   Bids: {len(book.get('bids', []))}")
        print(f"   Asks: {len(book.get('asks', []))}")

        print("\n🎉 API 密钥验证成功！")
        print("\n下一步:")
        print("  1. 检查 Rust 实现的 L2 签名是否正确")
        print("  2. 对比 Python SDK 和 Rust 实现的差异")

    except Exception as e:
        print(f"❌ 测试失败：{e}")
        print("\n可能原因:")
        print("  1. API 密钥是测试网的，不是主网的")
        print("  2. API 密钥已过期或被撤销")
        print("  3. 网络连接问题")
        sys.exit(1)


if __name__ == "__main__":
    main()
