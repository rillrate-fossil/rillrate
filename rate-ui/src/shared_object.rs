use crate::storage::typed_storage::{Storable, TypedStorage};
use crate::widget::{Context, NotificationHandler};
use std::cell::{Ref, RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use typed_slab::TypedSlab;
use yew::Callback;

pub trait RouterState: PartialEq + Storable + Default + Clone + 'static {
    /// Called when the state restored.
    ///
    /// For the case when some data should be reset of expired.
    fn restored(&mut self);

    // TODO: Remove it completely!
    fn update(&mut self, new_state: Self) {
        *self = new_state;
    }

    fn on_update(&mut self) {}
}

pub struct DataChanged<T> {
    _data: PhantomData<T>,
}

impl<T> DataChanged<T> {
    fn new() -> Self {
        Self { _data: PhantomData }
    }
}

#[derive(Debug)]
pub struct SharedObject<T> {
    inner: Rc<RefCell<SharedObjectInner<T>>>,
    listener: Option<ListenerId>,
}

impl<T> Clone for SharedObject<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            listener: None,
        }
    }
}

impl<T> Drop for SharedObject<T> {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

impl<T> Default for SharedObject<T>
where
    T: RouterState + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SharedObject<T> {
    pub fn new() -> Self
    where
        T: RouterState + Default,
    {
        let storage = TypedStorage::new();
        let mut data: T = storage.restore().unwrap_or_default();
        data.restored();
        let inner = SharedObjectInner {
            callbacks: TypedSlab::new(),
            data,
        };
        Self {
            inner: Rc::new(RefCell::new(inner)),
            listener: None,
        }
    }

    pub fn subscribe<W>(&mut self, context: &mut Context<W>)
    where
        W: NotificationHandler<DataChanged<T>>,
        T: 'static,
    {
        self.unsubscribe();
        let callback = context.notification();
        let mut writer = self.inner.borrow_mut();
        let id = writer.callbacks.insert(callback);
        self.listener = Some(id);
    }

    pub fn unsubscribe(&mut self) {
        if let Some(id) = self.listener.take() {
            let mut writer = self.inner.borrow_mut();
            writer.callbacks.remove(id);
        }
    }

    pub fn read(&self) -> SharedObjectReader<'_, T> {
        let reader = self.inner.borrow();
        SharedObjectReader { reader }
    }

    pub fn write(&self) -> SharedObjectWriter<'_, T>
    where
        T: RouterState,
    {
        let writer = self.inner.borrow_mut();
        SharedObjectWriter { writer }
    }
}

pub struct SharedObjectReader<'a, T> {
    reader: Ref<'a, SharedObjectInner<T>>,
}

impl<'a, T> Deref for SharedObjectReader<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.reader.data
    }
}

pub struct SharedObjectWriter<'a, T: RouterState> {
    writer: RefMut<'a, SharedObjectInner<T>>,
}

impl<'a, T> Deref for SharedObjectWriter<'a, T>
where
    T: RouterState,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.writer.data
    }
}

impl<'a, T> DerefMut for SharedObjectWriter<'a, T>
where
    T: RouterState,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer.data
    }
}

impl<'a, T> Drop for SharedObjectWriter<'a, T>
where
    T: RouterState,
{
    fn drop(&mut self) {
        self.writer.data.on_update();
        // TODO: Avoid creating it every time. Or is it cheap?
        let mut storage = TypedStorage::new();
        storage.store(&self.writer.data);
        for (_, callback) in self.writer.callbacks.iter() {
            callback.emit(DataChanged::new());
        }
    }
}

type ListenerId = usize;

#[derive(Debug)]
struct SharedObjectInner<T> {
    callbacks: TypedSlab<ListenerId, Callback<DataChanged<T>>>,
    data: T,
}
