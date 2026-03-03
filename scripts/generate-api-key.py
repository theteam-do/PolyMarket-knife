#!/usr/bin/env python3
"""
Polymarket CLOB API 密钥生成脚本
用于 PolyMarket Knife 项目

用法：
    python3 generate-api-key.py

输出：
    API Key, Secret, Passphrase - 用于配置 market-maker
"""

import sys
import json

try:
    from py_clob_client.client import ClobClient
    from py_clob_client.config import ClobApiParams
except ImportError:
    print("❌ 缺少依赖：py-clob-client")
    print("\n请安装:")
    print("  pip3 install py-clob-client")
    print("\n或使用虚拟环境:")
    print("  python3 -m venv venv")
    print("  source venv/bin/activate")
    print("  pip3 install py-clob-client")
    sys.exit(1)

# 测试钱包私钥（PolyMarket Knife 测试用）
PRIVATE_KEY = "0x18f0d0ca93a73f451cf42ea17bf4cae1286fd352f81f1a965650ea49fb5951e7"
WALLET_ADDRESS = "0x8188D941E07de699c16e1D5eb098ad62FAd6B3e6"
CLOB_HOST = "https://clob.polymarket.com"
CHAIN_ID = 137  # Polygon 主网


def main():
    print("🔑 Polymarket CLOB API 密钥生成工具")
    print("=" * 50)
    print(f"钱包地址：{WALLET_ADDRESS}")
    print(f"CLOB 主机：{CLOB_HOST}")
    print(f"网络：Polygon 主网 (Chain ID: {CHAIN_ID})")
    print("=" * 50)
    print()

    try:
        # 创建 CLOB 客户端
        print("📡 连接 CLOB API...")
        client = ClobClient(host=CLOB_HOST, chain_id=CHAIN_ID, key=PRIVATE_KEY)
        print("✅ 连接成功")
        print()

        # 创建或派生 API 凭证
        print("🔐 创建/派生 API 凭证...")
        creds = client.create_or_derive_api_creds()

        api_key = creds.get("apiKey", "")
        api_secret = creds.get("secret", "")
        passphrase = creds.get("passphrase", "")

        if not all([api_key, api_secret, passphrase]):
            print("❌ API 凭证生成失败")
            sys.exit(1)

        print("✅ API 凭证生成成功!")
        print()

        # 显示结果
        print("┌" + "─" * 58 + "┐")
        print("│ 📋 API 凭证 (请立即保存!)                           │")
        print("├" + "─" * 58 + "┤")
        print(f"│ API Key:      {api_key:<40} │")
        print(f"│ API Secret:   {api_secret:<40} │")
        print(f"│ Passphrase:   {passphrase:<40} │")
        print("└" + "─" * 58 + "┘")
        print()

        # 生成配置文件片段
        print("📝 配置文件片段 (添加到 config/market-maker-mainnet-test.toml):")
        print("-" * 50)
        print("[clob]")
        print(f'api_key = "{api_key}"')
        print(f'api_secret = "{api_secret}"')
        print("-" * 50)
        print()

        # 生成环境变量命令
        print("📝 或使用环境变量:")
        print("-" * 50)
        print(f'export CLOB_API_KEY="{api_key}"')
        print(f'export CLOB_API_SECRET="{api_secret}"')
        print("-" * 50)
        print()

        # 保存到文件
        save_file = "api-credentials.json"
        with open(save_file, "w") as f:
            json.dump(
                {
                    "apiKey": api_key,
                    "secret": api_secret,
                    "passphrase": passphrase,
                    "wallet": WALLET_ADDRESS,
                    "chain_id": CHAIN_ID,
                },
                f,
                indent=2,
            )

        print(f"💾 凭证已保存到：{save_file}")
        print()

        # 安全提醒
        print("⚠️  安全提醒:")
        print("  1. 不要将 api-credentials.json 提交到 Git")
        print("  2. 已将文件添加到 .gitignore")
        print("  3. 测试完成后请删除此文件")
        print()

        # 添加到 .gitignore
        try:
            with open(".gitignore", "a") as f:
                f.write("\n# API 凭证\napi-credentials.json\n")
            print("✅ 已更新 .gitignore")
        except Exception as e:
            print(f"⚠️  无法更新 .gitignore: {e}")

        print()
        print("🎉 完成！现在可以运行 market-maker 了!")
        print()
        print("下一步:")
        print("  1. 复制上面的配置到 config/market-maker-mainnet-test.toml")
        print("  2. 上传配置文件到 VPS")
        print("  3. 运行：./market-maker config/market-maker-mainnet-test.toml")

    except Exception as e:
        print(f"❌ 错误：{e}")
        print()
        print("可能的原因:")
        print("  1. 网络连接问题")
        print("  2. 私钥格式错误")
        print("  3. 钱包从未登录过 Polymarket")
        print()
        print("解决方案:")
        print("  1. 先访问 https://polymarket.com 用钱包登录一次")
        print("  2. 检查私钥是否正确")
        print("  3. 稍后重试")
        sys.exit(1)


if __name__ == "__main__":
    main()
