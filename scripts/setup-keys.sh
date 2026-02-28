#!/bin/bash
# 密钥管理脚本

set -e

echo "======================================"
echo "  PolyMarket Knife Key Management"
echo "======================================"

# 检查参数
if [ -z "$1" ]; then
    echo "Usage: $0 <action>"
    echo "  Actions:"
    echo "    generate  - Generate new key pair"
    echo "    validate  - Validate existing key"
    echo "    backup    - Backup keys securely"
    echo "    rotate    - Rotate API keys"
    exit 1
fi

ACTION=$1

case $ACTION in
    generate)
        echo ""
        echo "Generating new key pair..."
        echo "--------------------------------------"
        
        # 生成私钥 (使用 openssl)
        PRIVATE_KEY=$(openssl rand -hex 32)
        echo "Private Key: 0x$PRIVATE_KEY"
        echo ""
        
        # 从私钥推导地址 (需要 web3 工具)
        echo "⚠️  Use web3 tools to derive address from private key"
        echo ""
        
        # 保存到安全位置
        echo "Save this key securely!"
        echo "Recommended: AWS Secrets Manager, HashiCorp Vault"
        ;;
        
    validate)
        echo ""
        echo "Validating key..."
        echo "--------------------------------------"
        
        if [ -z "$POLYMARKET_PRIVATE_KEY" ]; then
            echo "❌ POLYMARKET_PRIVATE_KEY not set"
            exit 1
        fi
        
        # 验证格式
        if [[ ! "$POLYMARKET_PRIVATE_KEY" =~ ^0x[a-fA-F0-9]{64}$ ]]; then
            echo "❌ Invalid private key format"
            echo "Expected: 0x followed by 64 hex characters"
            exit 1
        fi
        
        echo "✅ Private key format is valid"
        ;;
        
    backup)
        echo ""
        echo "Backing up keys..."
        echo "--------------------------------------"
        
        BACKUP_DIR="$HOME/.polymarket-knife/backup-$(date +%Y%m%d%H%M%S)"
        mkdir -p "$BACKUP_DIR"
        
        # 备份配置文件 (不含私钥)
        if [ -d "config" ]; then
            cp config/*.toml "$BACKUP_DIR/" 2>/dev/null || true
            # 移除私钥
            sed -i '/private_key/d' "$BACKUP_DIR"/*.toml 2>/dev/null || true
        fi
        
        echo "✅ Config backed up to: $BACKUP_DIR"
        echo "⚠️  Private keys are NOT backed up automatically"
        echo "   Use secure key management solution"
        ;;
        
    rotate)
        echo ""
        echo "Rotating API keys..."
        echo "--------------------------------------"
        echo "⚠️  API key rotation must be done on Polymarket platform"
        echo ""
        echo "Steps:"
        echo "1. Login to Polymarket"
        echo "2. Go to API settings"
        echo "3. Generate new API key"
        echo "4. Update environment variables"
        echo "5. Test with new key"
        echo "6. Revoke old key"
        ;;
        
    *)
        echo "Unknown action: $ACTION"
        exit 1
        ;;
esac

echo ""
echo "======================================"
echo "  Security Best Practices"
echo "======================================"
echo "1. Never commit private keys to git"
echo "2. Use environment variables or secret manager"
echo "3. Rotate keys regularly"
echo "4. Monitor key usage"
echo "5. Use separate keys for different environments"
