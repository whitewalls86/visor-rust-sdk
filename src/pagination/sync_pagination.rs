use std::collections::VecDeque;

use crate::client::VisorClient;
use crate::error::VisorError;
use crate::models::dealers::{DealerFilter, DealerSummary};
use crate::models::listings::{ListingSummary, ListingsFilter};

struct ListingsIter<'a> {
    client: &'a VisorClient,
    filter: ListingsFilter,
    buffer: VecDeque<ListingSummary>,
    pages_fetched: usize,
    max_pages: Option<usize>,
    done: bool,
}

impl<'a> Iterator for ListingsIter<'a> {
    type Item = Result<ListingSummary, VisorError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.buffer.pop_front() {
            return Some(Ok(item));
        }
        if self.done {
            return None;
        }
        if self.max_pages.is_some_and(|max| self.pages_fetched >= max) {
            return None;
        }
        match self.client.filter_listings(&self.filter) {
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
            Ok(page) => {
                self.pages_fetched += 1;
                if page.data.is_empty() {
                    self.done = true;
                    return None;
                }
                advance_offset(
                    &mut self.done,
                    &mut self.filter.offset,
                    page.pagination.next_offset,
                );
                self.buffer = page.data.into_iter().collect();
                self.buffer.pop_front().map(Ok)
            }
        }
    }
}

/// Returns an iterator over listing summaries from paginated API responses.
///
/// Fetches pages until the API is exhausted or `max_pages` pages have been
/// fetched. `Some(0)` performs no request. `None` means no page limit.
pub fn iter_listings(
    client: &VisorClient,
    filter: ListingsFilter,
    max_pages: Option<usize>,
) -> impl Iterator<Item = Result<ListingSummary, VisorError>> + '_ {
    ListingsIter {
        client,
        filter,
        buffer: VecDeque::new(),
        pages_fetched: 0,
        max_pages,
        done: false,
    }
}

struct DealersIter<'a> {
    client: &'a VisorClient,
    filter: DealerFilter,
    buffer: VecDeque<DealerSummary>,
    pages_fetched: usize,
    max_pages: Option<usize>,
    done: bool,
}

impl<'a> Iterator for DealersIter<'a> {
    type Item = Result<DealerSummary, VisorError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.buffer.pop_front() {
            return Some(Ok(item));
        }
        if self.done {
            return None;
        }
        if self.max_pages.is_some_and(|max| self.pages_fetched >= max) {
            return None;
        }
        match self.client.search_dealers(&self.filter) {
            Err(e) => {
                self.done = true;
                Some(Err(e))
            }
            Ok(page) => {
                self.pages_fetched += 1;
                if page.data.is_empty() {
                    self.done = true;
                    return None;
                }
                advance_offset(
                    &mut self.done,
                    &mut self.filter.offset,
                    page.pagination.next_offset,
                );
                self.buffer = page.data.into_iter().collect();
                self.buffer.pop_front().map(Ok)
            }
        }
    }
}

/// Returns an iterator over dealer summaries from paginated API responses.
///
/// Fetches pages until the API is exhausted or `max_pages` pages have been
/// fetched. `Some(0)` performs no request. `None` means no page limit.
pub fn iter_dealers(
    client: &VisorClient,
    filter: DealerFilter,
    max_pages: Option<usize>,
) -> impl Iterator<Item = Result<DealerSummary, VisorError>> + '_ {
    DealersIter {
        client,
        filter,
        buffer: VecDeque::new(),
        pages_fetched: 0,
        max_pages,
        done: false,
    }
}

/// Updates `current_offset` to `next_offset` when it advances, or sets `done`
/// if `next_offset` is absent or non-advancing (guards against infinite loops).
fn advance_offset(done: &mut bool, current_offset: &mut u32, next_offset: Option<i32>) {
    match next_offset {
        Some(next) if next >= 0 && (next as u32) > *current_offset => {
            *current_offset = next as u32;
        }
        _ => *done = true,
    }
}
