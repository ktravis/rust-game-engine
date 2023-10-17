use crate::renderer::{InstanceData, RawInstanceData};

pub trait Batcher: Default {
    type Item: InstanceData;
    fn add(&mut self, instance_data: Self::Item);
    fn active_instances(&self) -> &[Self::Item];
    fn instance_count(&self) -> usize;
    fn full(&self) -> bool;
    fn clear(&mut self);
}

#[derive(Debug, Copy, Clone, Default)]
pub struct NoopBatcher(bool);

impl Batcher for NoopBatcher {
    type Item = ();

    fn add(&mut self, _instance_data: Self::Item) {
        self.0 = true;
    }

    #[inline]
    fn active_instances(&self) -> &[Self::Item] {
        if self.0 {
            &[()]
        } else {
            &[]
        }
    }

    #[inline]
    fn instance_count(&self) -> usize {
        if self.0 {
            1
        } else {
            0
        }
    }

    #[inline]
    fn full(&self) -> bool {
        self.0
    }

    #[inline]
    fn clear(&mut self) {
        self.0 = false;
    }
}

// TODO add "render params" to compare against
#[derive(Clone)]
pub struct RenderBatch<I: InstanceData = RawInstanceData, const MAX_INSTANCES: usize = 128>
where
    I: Sized,
{
    next_instance: usize,
    instances: [I; MAX_INSTANCES],
}

impl<I: InstanceData, const N: usize> Default for RenderBatch<I, N> {
    fn default() -> Self {
        Self {
            next_instance: 0,
            instances: [Default::default(); N],
        }
    }
}

impl<I: InstanceData, const N: usize> Batcher for RenderBatch<I, N> {
    type Item = I;

    fn add(&mut self, instance_data: I) {
        self.instances[self.next_instance] = instance_data;
        self.next_instance += 1;
    }

    #[inline]
    fn active_instances(&self) -> &[I] {
        &self.instances[..self.next_instance]
    }

    #[inline]
    fn instance_count(&self) -> usize {
        self.next_instance
    }

    #[inline]
    fn full(&self) -> bool {
        self.next_instance == N
    }

    #[inline]
    fn clear(&mut self) {
        self.next_instance = 0;
    }
}
