#![no_std]
#![forbid(clippy::undocumented_unsafe_blocks)]

//! See [`LazyPinned`].

use core::pin::Pin;

/// Pinned data which can be lazily initialized.
///
/// ## [`LazyPinned<T>`] vs. [`Option<T>`]
///
/// [`LazyPinned<T>`] act like [`Option<T>`].
/// In fact, `LazyPinned<T>` is implemented by just wrapping `Option<T>`.
/// However, they have different behaviors in pinning projection.
///
/// `Pin<P<Option<T>>>` guarantees the `Option<T>` is not moved,
/// where `P<_>` is a pointer type which deref to `_`.
/// Thus, when the data is `None`, it cannot be set to `Some(T)` unless
/// `T: Unpin`.
///
/// `Pin<P<LazyPinned<T>>>` only guarantees the inner `T` is pinned.
/// Thus, `Pin<&mut LazyPinned<T>>` optionally projects to `Pin<&mut T>`
/// instead of `Pin<P<Option<T>>>`. When `Pin<P<LazyPinned<T>>>` is `None`,
/// it can be initialized with a value of `T`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LazyPinned<T>(pub Option<T>);

impl<T> Default for LazyPinned<T> {
    #[inline]
    fn default() -> Self {
        Self(None)
    }
}

impl<T> LazyPinned<T> {
    #[inline]
    #[must_use]
    pub fn as_pin_ref(self: Pin<&Self>) -> Option<Pin<&T>> {
        Pin::get_ref(self).0.as_ref().map(|x| {
            // SAFETY: `x` is guaranteed to be pinned because it comes from `self`
            // which is pinned.
            unsafe { Pin::new_unchecked(x) }
        })
    }

    #[inline]
    #[must_use]
    pub fn as_pin_mut(self: Pin<&mut Self>) -> Option<Pin<&mut T>> {
        // SAFETY: `get_unchecked_mut` is never used to move the `Option` inside `self`.
        // `x` is guaranteed to be pinned because it comes from `self` which is pinned.
        unsafe {
            Pin::get_unchecked_mut(self)
                .0
                .as_mut()
                .map(|x| Pin::new_unchecked(x))
        }
    }

    pub fn pin_project_or_insert(self: Pin<&mut Self>, v: T) -> Pin<&mut T> {
        // SAFETY: `get_unchecked_mut` is never used to move the `Some(T)` inside `self`.
        let this = unsafe { Pin::get_unchecked_mut(self) };
        let x = this.0.get_or_insert(v);
        // SAFETY: `x` is guaranteed to be pinned because it comes from `self` which is pinned.
        unsafe { Pin::new_unchecked(x) }
    }

    pub fn pin_project_or_insert_with(self: Pin<&mut Self>, f: impl FnOnce() -> T) -> Pin<&mut T> {
        // SAFETY: `get_unchecked_mut` is never used to move the `Some(T)` inside `self`.
        let this = unsafe { Pin::get_unchecked_mut(self) };
        let x = this.0.get_or_insert_with(f);
        // SAFETY: `x` is guaranteed to be pinned because it comes from `self` which is pinned.
        unsafe { Pin::new_unchecked(x) }
    }

    pub fn use_pin_or_insert(
        self: Pin<&mut Self>,
        use_pin: impl FnOnce(Pin<&mut T>),
        v: T,
    ) -> Pin<&mut T> {
        self.use_pin_or_insert_with(use_pin, move || v)
    }

    pub fn use_pin_or_insert_with(
        self: Pin<&mut Self>,
        use_pin: impl FnOnce(Pin<&mut T>),
        insert: impl FnOnce() -> T,
    ) -> Pin<&mut T> {
        // SAFETY: `get_unchecked_mut` is never used to move the `Some(T)` inside `self`.
        let this = unsafe { Pin::get_unchecked_mut(self) };

        match &mut this.0 {
            Some(x) => {
                // SAFETY: `x` is guaranteed to be pinned because it comes from `self` which is pinned.
                let mut x = unsafe { Pin::new_unchecked(x) };
                use_pin(x.as_mut());
                x
            }
            this @ None => {
                let x = this.insert(insert());

                // SAFETY: `x` is guaranteed to be pinned because it comes from `self` which is pinned.
                unsafe { Pin::new_unchecked(x) }
            }
        }
    }

    pub fn use_pin_or_insert_with_data<Data>(
        self: Pin<&mut Self>,
        data: Data,
        use_pin: impl FnOnce(Data, Pin<&mut T>),
        insert: impl FnOnce(Data) -> T,
    ) -> Pin<&mut T> {
        // SAFETY: `get_unchecked_mut` is never used to move the `Some(T)` inside `self`.
        let this = unsafe { Pin::get_unchecked_mut(self) };

        match &mut this.0 {
            Some(x) => {
                // SAFETY: `x` is guaranteed to be pinned because it comes from `self` which is pinned.
                let mut x = unsafe { Pin::new_unchecked(x) };
                use_pin(data, x.as_mut());
                x
            }
            this @ None => {
                let x = this.insert(insert(data));

                // SAFETY: `x` is guaranteed to be pinned because it comes from `self` which is pinned.
                unsafe { Pin::new_unchecked(x) }
            }
        }
    }
}
