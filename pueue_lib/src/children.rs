use std::collections::BTreeMap;

use command_group::GroupChild;

/// This structure is needed to manage worker pools for groups.
/// It's a newtype pattern around a nested BTreeMap, which implements some convenience functions.
///
/// The datastructure represents the following data:
/// BTreeMap<group_name, BTreeMap<group_worker_id, (task_id, subprocess_handle)>
#[derive(Debug, Default)]
pub struct Children(pub BTreeMap<String, BTreeMap<usize, (usize, GroupChild)>>);

impl Children {
    /// Returns whether there are any active tasks across all groups.
    pub fn has_active_tasks(&self) -> bool {
        self.0.iter().any(|(_, pool)| !pool.is_empty())
    }

    /// Returns whether there are any active tasks for the given group.
    ///
    /// Returns `false` if the group cannot be found.
    pub fn has_group_active_tasks(&self, group: &str) -> bool {
        self.0
            .get(group)
            .map(|pool| !pool.is_empty())
            .unwrap_or(false)
    }

    /// A convenience function to check whether there's child with a given task_id.
    /// We have to do a nested linear search, as these datastructure aren't indexed via task_ids.
    pub fn has_child(&self, task_id: usize) -> bool {
        for pool in self.0.values() {
            for (child_task_id, _) in pool.values() {
                if child_task_id == &task_id {
                    return true;
                }
            }
        }

        false
    }

    /// A convenience function to get a mutable child by its respective task_id.
    /// We have to do a nested linear search over all children of all pools,
    /// beceause these datastructure aren't indexed via task_ids.
    pub fn get_child_mut(&mut self, task_id: usize) -> Option<&mut GroupChild> {
        for pool in self.0.values_mut() {
            for (child_task_id, child) in pool.values_mut() {
                if child_task_id == &task_id {
                    return Some(child);
                }
            }
        }

        None
    }

    /// A convenience function to get a list with all task_ids of all children.
    pub fn all_task_ids(&self) -> Vec<usize> {
        let mut task_ids = Vec::new();
        for pool in self.0.values() {
            for (task_id, _) in pool.values() {
                task_ids.push(*task_id)
            }
        }

        task_ids
    }

    /// Returns the next free worker slot for a given group.
    /// This function doesn't take Pueue's configuration into account, it simply returns the next
    /// free integer key, starting from 0.
    ///
    /// This function should only be called when spawning a new process.
    /// At this point, we're sure that the worker pool for the given group already exists, hence
    /// the expect call.
    pub fn get_next_group_worker(&self, group: &str) -> usize {
        let pool = self
            .0
            .get(group)
            .expect("The worker pool should be initialized when getting the next worker id.");

        // This does a simple linear scan over the worker keys of the process group.
        // Keys in a BTreeMap are ordered, which is why we can start at 0 and check for each entry,
        // if it is the same as our current id.
        //
        // E.g. If all slots to the last are full, we should walk through all keys and increment by
        // one each time.
        // If the second slot is free, we increment once, break the loop in the second iteration
        // and return the new id.
        let mut next_worker_id = 0;
        for worker_id in pool.keys() {
            if worker_id != &next_worker_id {
                break;
            }
            next_worker_id += 1;
        }

        next_worker_id
    }

    /// Inserts a new children into the worker pool of the given group.
    ///
    /// This function should only be called when spawning a new process.
    /// At this point, we're sure that the worker pool for the given group already exists, hence
    /// the expect call.
    pub fn add_child(&mut self, group: &str, worker_id: usize, task_id: usize, child: GroupChild) {
        let pool = self
            .0
            .get_mut(group)
            .expect("The worker pool should be initialized when inserting a new child.");

        pool.insert(worker_id, (task_id, child));
    }
}
