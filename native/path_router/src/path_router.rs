pub struct Tree<'a, T> {
    label: &'a str,
    value: Option<(T, Vec<&'a str>)>,
    branches: Vec<Tree<'a, T>>,
}

impl<'a, T> Tree<'a, T> {
    pub fn new<'b>() -> Tree<'b, T> {
        Tree {
            label: "",
            value: None,
            branches: Vec::new(),
        }
    }

    pub fn add(&mut self, key: &'a str, value: T) {
        let segments = key.split('/').filter(|s| !s.is_empty());
        let capture_labels = Vec::new();
        self.add_(segments, value, capture_labels);
    }

    fn add_<I: Iterator<Item = &'a str>>(
        &mut self,
        mut segments: I,
        value: T,
        mut capture_labels: Vec<&'a str>,
    ) {
        match segments.next() {
            None => {
                if self.value.is_some() {
                    panic!("Duplicate route!");
                }
                self.value = Some((value, capture_labels));
            }
            Some(segment) => {
                if let Some(existing_branch) = self.branches.iter_mut().find(|t| t.label == segment)
                {
                    existing_branch.add_(segments, value, capture_labels);
                    return;
                }
                if segment.starts_with(':') {
                    capture_labels.push(&segment[1..]);
                    if let Some(existing_branch) =
                        self.branches.iter_mut().find(|t| t.label.is_empty())
                    {
                        existing_branch.add_(segments, value, capture_labels);
                        return;
                    }
                    let mut branch = Tree {
                        label: "",
                        value: None,
                        branches: Vec::new(),
                    };
                    branch.add_(segments, value, capture_labels);
                    self.branches.push(branch);
                } else {
                    let mut branch = Tree {
                        label: segment,
                        value: None,
                        branches: Vec::new(),
                    };
                    branch.add_(segments, value, capture_labels);
                    self.branches.push(branch);
                }
            }
        }
    }

    pub fn find<'b>(&self, key: &'b str) -> Option<(&T, Vec<(&'a str, &'b str)>)> {
        let segments: Vec<&str> = key.split('/').filter(|s| !s.is_empty()).collect();
        let mut captured = Vec::new();
        self.find_(segments.as_slice(), &mut captured)
            .map(|(v, labels)| (v, labels.iter().cloned().zip(captured).collect()))
    }

    fn find_<'b>(
        &self,
        segments: &[&'b str],
        captured: &mut Vec<&'b str>,
    ) -> Option<(&T, Vec<&'a str>)> {
        match segments.split_first() {
            None => self.value.as_ref().map(|(v, labels)| (v, labels.clone())),
            Some((&segment, remaining)) => {
                // Try to find an exact match first
                if let Some(exact_match) = self
                    .branches
                    .iter()
                    .find(|t| t.label == segment)
                    .and_then(|t| t.find_(remaining, captured))
                {
                    return Some(exact_match);
                }

                // Try to find a prefix match
                if let Some(prefix_match) =
                    self.branches.iter().find(|t| t.label == "").and_then(|t| {
                        captured.push(segment);
                        let result = t.find_(remaining, captured);
                        if result.is_none() {
                            captured.pop();
                        }
                        result
                    })
                {
                    return Some(prefix_match);
                }

                // If no match found in branches, return the current value if any
                if let Some((value, captures)) = &self.value {
                    let mut captures = captures.clone();
                    let remaining_path = std::iter::once(segment)
                        .chain(remaining.iter().cloned())
                        .collect::<Vec<_>>()
                        .join("/");
                    if !remaining_path.is_empty() {
                        captures.push("_remaining");
                        captured.push(Box::leak(Box::new(remaining_path)).as_str());
                    }
                    return Some((value, captures));
                }

                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tree;

    #[test]
    fn can_add_and_find() {
        let mut routes = Tree::new();
        routes.add("/", 0);
        routes.add("/var", 1);
        routes.add("/var/www", 11);
        routes.add("/var/log", 12);
        assert_eq!(routes.find("/vax"), None);
        assert_eq!(routes.find("/var/something"), None);
        assert_eq!(routes.find("////"), Some((&0, vec![])));
        assert_eq!(routes.find("//var//"), Some((&1, vec![])));
        assert_eq!(routes.find("/var/www/"), Some((&11, vec![])));
        assert_eq!(routes.find("/var/log/"), Some((&12, vec![])));
    }

    #[test]
    fn can_add_and_capture_and_find() {
        let mut routes = Tree::new();
        routes.add("/user/:username", 11);
        routes.add("/user/:username/:intent/show", 111);
        routes.add("/user/:username/profile", 112);
        assert_eq!(routes.find("/user/myname/delete"), None);
        assert_eq!(routes.find("/user/myname/cook/throw"), None);
        assert_eq!(
            routes.find("/user/myname"),
            Some((&11, vec![("username", "myname")]))
        );
        assert_eq!(
            routes.find("/user/myname/profile"),
            Some((&112, vec![("username", "myname")]))
        );
        assert_eq!(
            routes.find("/user/myname/cook/show"),
            Some((&111, vec![("username", "myname"), ("intent", "cook")]))
        );
    }

    #[test]
    fn can_match_longest_prefix_with_remaining() {
        let mut routes = Tree::new();
        routes.add("/a", 1);
        routes.add("/a/b", 2);
        routes.add("/a/b/c", 3);
        assert_eq!(routes.find("/a"), Some((&1, vec![])));
        assert_eq!(routes.find("/a/b"), Some((&2, vec![])));
        assert_eq!(routes.find("/a/b/c"), Some((&3, vec![])));
        assert_eq!(
            routes.find("/a/b/c/d"),
            Some((&3, vec![("_remaining", "d")]))
        );
        assert_eq!(
            routes.find("/a/b/x"),
            Some((&2, vec![("_remaining", "b/x")]))
        );
        assert_eq!(
            routes.find("/a/b/c/d/e/f"),
            Some((&3, vec![("_remaining", "c/d/e/f")]))
        );
    }
}
