use temci::run::cpuset::CpuSet;

#[test]
fn test_cpuset_empty() {
    let cpuset = CpuSet::empty();
    assert_eq!(cpuset.len(), 0);
    assert!(cpuset.is_empty());
    assert!(!cpuset.contains(0));
}

#[test]
fn test_cpuset_single() {
    let cpuset = CpuSet::from_cpu(0);
    assert_eq!(cpuset.len(), 1);
    assert!(cpuset.contains(0));
    assert!(!cpuset.contains(1));
}

#[test]
fn test_cpuset_from_vec() {
    let cpuset = CpuSet::from_cpus(&[0, 2, 4]);
    assert_eq!(cpuset.len(), 3);
    assert!(cpuset.contains(0));
    assert!(cpuset.contains(2));
    assert!(cpuset.contains(4));
    assert!(!cpuset.contains(1));
}

#[test]
fn test_cpuset_range() {
    let cpuset = CpuSet::from_range(0, 3);
    assert_eq!(cpuset.len(), 3);
    assert!(cpuset.contains(0));
    assert!(cpuset.contains(1));
    assert!(cpuset.contains(2));
    assert!(!cpuset.contains(3));
}

#[test]
fn test_cpuset_add() {
    let mut cpuset = CpuSet::empty();
    cpuset.add(0);
    cpuset.add(2);
    assert_eq!(cpuset.len(), 2);
    assert!(cpuset.contains(0));
    assert!(cpuset.contains(2));
}

#[test]
fn test_cpuset_add_range() {
    let mut cpuset = CpuSet::empty();
    cpuset.add_range(1, 3);
    assert_eq!(cpuset.len(), 2);
    assert!(cpuset.contains(1));
    assert!(cpuset.contains(2));
}

#[test]
fn test_cpuset_remove() {
    let mut cpuset = CpuSet::from_cpus(&[0, 1, 2]);
    cpuset.remove(1);
    assert_eq!(cpuset.len(), 2);
    assert!(cpuset.contains(0));
    assert!(!cpuset.contains(1));
    assert!(cpuset.contains(2));
}

#[test]
fn test_cpuset_clear() {
    let mut cpuset = CpuSet::from_cpus(&[0, 1, 2]);
    cpuset.clear();
    assert_eq!(cpuset.len(), 0);
    assert!(cpuset.is_empty());
}

#[test]
fn test_cpuset_iter() {
    let cpuset = CpuSet::from_cpus(&[2, 0, 4]);
    let cpus: Vec<usize> = cpuset.iter().collect();
    assert_eq!(cpus, vec![0, 2, 4]); // Should be sorted
}

#[test]
fn test_cpuset_all() {
    let cpuset = CpuSet::all();
    // On a typical system, should have at least 1 CPU
    assert!(!cpuset.is_empty());
}

#[test]
fn test_cpuset_display() {
    let cpuset = CpuSet::from_cpus(&[0, 1, 2]);
    let s = format!("{}", cpuset);
    assert!(s.contains("0") || s.contains("1") || s.contains("2"));
}

#[test]
fn test_cpuset_parse_single() {
    let cpuset = CpuSet::parse("0").unwrap();
    assert_eq!(cpuset.len(), 1);
    assert!(cpuset.contains(0));
}

#[test]
fn test_cpuset_parse_list() {
    let cpuset = CpuSet::parse("0,2,4").unwrap();
    assert_eq!(cpuset.len(), 3);
    assert!(cpuset.contains(0));
    assert!(cpuset.contains(2));
    assert!(cpuset.contains(4));
}

#[test]
fn test_cpuset_parse_range() {
    let cpuset = CpuSet::parse("0-2").unwrap();
    assert_eq!(cpuset.len(), 3);
    assert!(cpuset.contains(0));
    assert!(cpuset.contains(1));
    assert!(cpuset.contains(2));
}

#[test]
fn test_cpuset_parse_mixed() {
    let cpuset = CpuSet::parse("0,2-4,6").unwrap();
    assert_eq!(cpuset.len(), 5); // 0, 2, 3, 4, 6
    assert!(cpuset.contains(0));
    assert!(cpuset.contains(2));
    assert!(cpuset.contains(3));
    assert!(cpuset.contains(4));
    assert!(cpuset.contains(6));
}

#[test]
fn test_cpuset_parse_invalid() {
    assert!(CpuSet::parse("").is_err());
    assert!(CpuSet::parse("abc").is_err());
}

#[test]
fn test_cpuset_first() {
    let cpuset = CpuSet::from_cpus(&[2, 0, 4]);
    assert_eq!(cpuset.first(), Some(0));
}

#[test]
fn test_cpuset_first_empty() {
    let cpuset = CpuSet::empty();
    assert_eq!(cpuset.first(), None);
}

#[test]
fn test_cpuset_count() {
    let cpuset = CpuSet::all();
    let count = cpuset.count();
    assert!(count > 0);
}

#[test]
fn test_cpuset_clone() {
    let cpuset1 = CpuSet::from_cpus(&[0, 1, 2]);
    let cpuset2 = cpuset1.clone();
    assert_eq!(cpuset1.len(), cpuset2.len());
    assert!(cpuset2.contains(0));
    assert!(cpuset2.contains(1));
    assert!(cpuset2.contains(2));
}
