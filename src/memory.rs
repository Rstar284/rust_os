use x86_64::structures::paging::{FrameAllocator, Mapper, mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB}; 
use x86_64::{PhysAddr, VirtAddr, registers::control::Cr3};
use x86_64::structures::paging::PageTableFlags as Flags;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicU64, Ordering};


pub unsafe fn init(phy_mem_off: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_l4_table(phy_mem_off);
    OffsetPageTable::new(l4_table, phy_mem_off)
}

unsafe fn active_l4_table(phy_mem_off: VirtAddr) -> &'static mut PageTable {
    let (l4, _) = Cr3::read();
    let phy = l4.start_address();
    let virt = phy_mem_off + phy.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

pub fn example_mapping(page: Page, mapper: &mut OffsetPageTable, frame_alloc: &mut impl FrameAllocator<Size4KiB>) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_res = unsafe {mapper.map_to(page, frame, flags, frame_alloc)};
    map_to_res.expect("map_to has failed").flush();
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0
        }
    }
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addr = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addr.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackBounds {
    start: VirtAddr,
    end: VirtAddr,
}

impl StackBounds {
    pub fn new(start: VirtAddr, end: VirtAddr) -> Self {
        assert!(end > start);
        StackBounds { start, end }
    }
    pub fn start(&self) -> VirtAddr {
        self.start
    }
    pub fn end(&self)-> VirtAddr {
        self.end
    }
}

fn reserve_stack_mem(size_p: u64) -> Page {
    static STACK_ALLOC_NEXT: AtomicU64 = AtomicU64::new(0x_5555_5555_0000);
    let start_addr = VirtAddr::new(STACK_ALLOC_NEXT.fetch_add(size_p * Page::<Size4KiB>::SIZE, Ordering::Relaxed));
    Page::from_start_address(start_addr).expect("STACK_ALLOC_NEXT is not aligned")
}

pub fn alloc_stack(size_p: u64, mapper: &mut impl Mapper<Size4KiB>, frame_alloc: &mut impl FrameAllocator<Size4KiB>) -> Result<StackBounds, mapper::MapToError<Size4KiB>> {
    let guard_page = reserve_stack_mem(size_p +1);
    let stack_start = guard_page + 1;
    let stack_end = stack_start + size_p;
    for page in Page::range(stack_start, stack_end) {
        let frame = frame_alloc.allocate_frame().ok_or(mapper::MapToError::FrameAllocationFailed)?;
        let flags = Flags::PRESENT | Flags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_alloc)?.flush(); }
    }
    Ok(StackBounds { start: stack_start.start_address(), end: stack_end.start_address() })
}