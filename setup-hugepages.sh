#!/usr/bin/env bash
set -e

# Amount of hugepages to allocate (default: 4GB)
# Each hugepage is 2MB, so 4GB = 2048 pages
HUGEPAGE_SIZE_MB=2
TOTAL_HUGEPAGES_GB=${HUGEPAGES_GB:-4}
NUM_HUGEPAGES=$((TOTAL_HUGEPAGES_GB * 1024 / HUGEPAGE_SIZE_MB))

echo "Setting up hugepages..."
echo "Allocating ${TOTAL_HUGEPAGES_GB}GB (${NUM_HUGEPAGES} pages of ${HUGEPAGE_SIZE_MB}MB each)"

# Allocate hugepages
echo $NUM_HUGEPAGES | sudo tee /sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages > /dev/null

# Create mount point if it doesn't exist
if [ ! -d /mnt/huge ]; then
    echo "Creating /mnt/huge directory..."
    sudo mkdir -p /mnt/huge
fi

# Mount hugetlbfs if not already mounted
if ! mount | grep -q hugetlbfs; then
    echo "Mounting hugetlbfs..."
    sudo mount -t hugetlbfs nodev /mnt/huge
else
    echo "hugetlbfs already mounted"
fi

# Verify allocation
ALLOCATED=$(cat /sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages)
FREE=$(cat /sys/kernel/mm/hugepages/hugepages-2048kB/free_hugepages)

echo ""
echo "Hugepages setup complete!"
echo "  Allocated: ${ALLOCATED} pages ($(($ALLOCATED * $HUGEPAGE_SIZE_MB / 1024))GB)"
echo "  Free: ${FREE} pages ($(($FREE * $HUGEPAGE_SIZE_MB / 1024))GB)"
echo ""
echo "Note: This is not persistent across reboots"
echo "      Run this script again after rebooting"
