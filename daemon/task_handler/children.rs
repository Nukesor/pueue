use std::collections::BTreeMap;
use std::process::Child;

/// This structure is needed to manage worker pools for groups.
/// It's a newtype pattern around a nested BTreeMap, which implements some convenience functions.
///
/// The datastructure contains these types of data:
/// BTreeMap<group, BTreeMap<group_worker_id, (task_id, Subprocess handle)>
pub struct Children(pub BTreeMap<String, BTreeMap<usize, (usize, Child)>>);

impl Children {
    /// Returns whether there are any active tasks across all groups.
    pub fn has_active_tasks(&self) -> bool {
        self.0.iter().any(|(_, pool)| !pool.is_empty())
    }

    /// A convenience function to check whether there's child with a given task_id.
    /// We have to do a nested linear search, as these datastructure aren't indexed via task_ids.
    pub fn has_child(&self, task_id: usize) -> bool {
        for (_, pool) in self.0.iter() {
            for (_, (child_task_id, _)) in pool.iter() {
                if child_task_id == &task_id {
                    return true;
                }
            }
        }

        false
    }

    /// A convenience function to get a child by its respective task_id.
    /// We have to do a nested linear search, as these datastructure aren't indexed via task_ids.
    pub fn get_child(&self, task_id: usize) -> Option<&Child> {
        for (_, pool) in self.0.iter() {
            for (_, (child_task_id, child)) in pool.iter() {
                if child_task_id == &task_id {
                    return Some(child);
                }
            }
        }

        None
    }

    /// A convenience function to get a child by its respective task_id.
    /// We have to do a nested linear search, as these datastructure aren't indexed via task_ids.
    pub fn get_child_mut(&mut self, task_id: usize) -> Option<&mut Child> {
        for (_, pool) in self.0.iter_mut() {
            for (_, (child_task_id, child)) in pool.iter_mut() {
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
        for (_, pool) in self.0.iter() {
            for (_, (task_id, _)) in pool.iter() {
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
    pub fn add_child(&mut self, group: &str, worker_id: usize, task_id: usize, child: Child) {
        let pool = self
            .0
            .get_mut(group)
            .expect("The worker pool should be initialized when inserting a new child.");

        pool.insert(worker_id, (task_id, child));
    }
}
