use crate::{combos::estimate_combos, config::WorkspaceConfig, manifest::Package};

pub fn select_chunk(
    total_chunks: usize,
    chunk: usize,
    packages: Vec<Package>,
    config: &WorkspaceConfig,
) -> Vec<Package> {
    assert!(chunk < total_chunks);

    let sorted = sort_by_combos(packages, config);
    let mut chunks = create_chunks(sorted, total_chunks);

    // Remove the chosen chunk and return it, dropping the rest.
    chunks.swap_remove(chunk)
}

/// Sorts a slice of [`Package`]s by the amount of feature combinations, based on
/// [`estimate_combos()`].
///
/// The returned [`Vec`] contains a tuples of the packages and their corresponding combinations. It
/// is sorted so that the package with the greatest amount of combinations will be last.
fn sort_by_combos(packages: Vec<Package>, config: &WorkspaceConfig) -> Vec<(Package, u128)> {
    let mut sorted = Vec::with_capacity(packages.len());

    // Calculate the amount of combos for each package, then add it to the list.
    for package in packages {
        let max_k = config
            .get(&package.name)
            .max_combo_size()
            .map(|k| k as u128);

        let combos = estimate_combos(package.features.len() as u128, max_k).unwrap();
        sorted.push((package, combos));
    }

    // Sort the list by the amount of combinations.
    // TODO: Investigate whether this should be stable or unstable, since deteriminism is required.
    sorted.sort_unstable_by_key(|(_, combo)| *combo);

    sorted
}

/// Creates a list of chunks from a list of packages sorted by their max amount of combinations.
fn create_chunks(mut sorted: Vec<(Package, u128)>, total_chunks: usize) -> Vec<Vec<Package>> {
    let mut chunks = vec_from_fn(Vec::new, total_chunks);
    let mut sizes = vec![0_u128; total_chunks];

    while let Some((package, combos)) = sorted.pop() {
        // Find the index of the chunk with the smallest size.
        let (i, _) = sizes
            .iter()
            .enumerate()
            .min_by_key(|(_, size)| **size)
            .unwrap();

        // Add the largest package to the smallest chunk, updating the size.
        chunks[i].push(package);
        sizes[i] = sizes[i].saturating_add(combos);
    }

    chunks
}

/// Creates a new [`Vec`] of a given length by calling the given closure for each element.
///
/// This is useful when initializing a [`Vec<T>`] where `T` does not implement [`Clone`].
fn vec_from_fn<T>(mut f: impl FnMut() -> T, len: usize) -> Vec<T> {
    let mut res = Vec::with_capacity(len);

    for _ in 0..len {
        res.push((f)());
    }

    res
}
