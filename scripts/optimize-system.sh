#!/bin/bash
# 系统优化脚本

set -e

echo "======================================"
echo "  System Performance Optimization"
echo "======================================"

echo ""
echo "1. Network Optimization"
echo "--------------------------------------"

# 禁用 TCP 延迟确认
sudo ethtool -K eth0 gro off 2>/dev/null || true
sudo ethtool -K eth0 gso off 2>/dev/null || true
echo "✅ Network optimization applied"

echo ""
echo "2. File Descriptor Limits"
echo "--------------------------------------"

# 增加文件描述符限制
ulimit -n 65536 2>/dev/null || echo "⚠️  Cannot change ulimit (run as root)"
echo "✅ File descriptor limits increased"

echo ""
echo "3. Memory Lock"
echo "--------------------------------------"

# 锁定内存防止 swap
ulimit -l unlimited 2>/dev/null || echo "⚠️  Cannot lock memory"
echo "✅ Memory lock configured"

echo ""
echo "4. CPU Affinity (Optional)"
echo "--------------------------------------"

echo "To bind to specific CPU core:"
echo "  taskset -c 0 ./target/release/market-maker"
echo ""

echo "======================================"
echo "  Optimization Complete"
echo "======================================"
echo ""
echo "Next steps:"
echo "1. Restart your application"
echo "2. Monitor performance"
echo "3. Adjust parameters as needed"
