#!/bin/bash
# 系统性能优化脚本

set -e

echo "======================================"
echo "  System Performance Optimization"
echo "======================================"

echo ""
echo "1. Network Optimization"
echo "--------------------------------------"

# 禁用 TCP 延迟确认
if command -v ethtool &> /dev/null; then
    sudo ethtool -K eth0 gro off 2>/dev/null || echo "⚠️  Cannot disable GRO"
    sudo ethtool -K eth0 gso off 2>/dev/null || echo "⚠️  Cannot disable GSO"
    sudo ethtool -K eth0 tso off 2>/dev/null || echo "⚠️  Cannot disable TSO"
    echo "✅ Network optimization applied"
else
    echo "⚠️  ethtool not available"
fi

echo ""
echo "2. File Descriptor Limits"
echo "--------------------------------------"

# 增加文件描述符限制
ulimit -n 65536 2>/dev/null && echo "✅ File descriptor limits increased" || echo "⚠️  Cannot change ulimit (run as root)"

echo ""
echo "3. Memory Lock"
echo "--------------------------------------"

# 锁定内存防止 swap
ulimit -l unlimited 2>/dev/null && echo "✅ Memory lock configured" || echo "⚠️  Cannot lock memory"

echo ""
echo "4. CPU Affinity (Optional)"
echo "--------------------------------------"

echo "To bind to specific CPU core:"
echo "  taskset -c 0 ./target/release/market-maker"
echo ""
echo "Or use numactl:"
echo "  numactl --cpunodebind=0 --membind=0 ./target/release/market-maker"

echo ""
echo "5. IRQ Affinity (Advanced)"
echo "--------------------------------------"

echo "To optimize network IRQ affinity:"
echo "  cat /proc/interrupts | grep eth0"
echo "  echo 1 > /proc/irq/<IRQ_NUMBER>/smp_affinity_list"

echo ""
echo "======================================"
echo "  Optimization Complete"
echo "======================================"
echo ""
echo "Next steps:"
echo "1. Restart your application"
echo "2. Monitor performance"
echo "3. Adjust parameters as needed"
echo ""
echo "Performance monitoring:"
echo "  watch -n 1 'cat /proc/$(pidof market-maker)/status | grep -E \"(VmRSS|Threads)\"'"
