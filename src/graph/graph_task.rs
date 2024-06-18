use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub struct Graph<'a> {
    pub independent_tasks: Vec<&'a Task>,
    nodes_dictionary: HashMap<String, usize>,
    edges: Vec<(usize, usize)>,
}

impl<'a> Graph<'a> {
    pub fn from_tasks_list(tasks_list: &'a [Task]) -> Self {
        let (independent_tasks, dependent_tasks) = Task::split_tasks(tasks_list);
        let mut nodes_dictionary: HashMap<String, usize> = HashMap::new();
        Self::populate_node_dictionary(&mut nodes_dictionary, &dependent_tasks);
        let edges = Self::build_edges(&dependent_tasks, &nodes_dictionary);
        Self {
            independent_tasks,
            nodes_dictionary,
            edges,
        }
    }

    pub fn nodes(&self) -> HashMap<&usize, &String> {
        self.nodes_dictionary
            .iter()
            .map(|node| (node.1, node.0))
            .collect()
    }

    pub fn edges(&self) -> Vec<(&usize, &usize)> {
        self.edges.iter().map(|t| (&t.0, &t.1)).collect()
    }

    fn build_edges(
        dependent_tasks: &[&Task],
        nodes_dictionary: &HashMap<String, usize>,
    ) -> Vec<(usize, usize)> {
        dependent_tasks
            .iter()
            .enumerate()
            .filter_map(|(uid, task)| {
                Self::dependecies_lists_to_tuple_nodes(&task.depends_on, uid, nodes_dictionary)
            })
            .flatten()
            .collect()
    }

    fn dependecies_lists_to_tuple_nodes(
        dependecies_lists: &Vec<String>,
        uid: usize,
        nodes_dictionary: &HashMap<String, usize>,
    ) -> Option<Vec<(usize, usize)>> {
        if dependecies_lists.is_empty() {
            return None;
        };
        let mut result: Vec<(usize, usize)> = vec![];
        for dependecy in dependecies_lists {
            match nodes_dictionary.get(dependecy) {
                Some(node) => result.push((*node, uid)),
                None => return None,
            }
        }
        Some(result)
    }

    fn populate_node_dictionary(
        nodes_dictionary: &mut HashMap<String, usize>,
        dependent_tasks: &[&Task],
    ) {
        dependent_tasks.iter().enumerate().for_each(|(uid, task)| {
            nodes_dictionary.insert(task.name.to_owned(), uid);
        });
    }

    pub fn format_independent_task(&self) -> String {
        //Format the indipendent tasks on the first line
        if self.independent_tasks.is_empty() {
            return String::new();
        };
        self.independent_tasks.iter().skip(1).fold(
            format!("|{}|", &self.independent_tasks[0].name),
            |accumulatotask_list, task| format!("{}    |{}|", accumulatotask_list, task.name),
        ) + "\n"
            + "\n"
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskFile {
    #[serde(flatten)]
    file: HashMap<String, DependsOn>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DependsOn {
    pub depends_on: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub name: String,

    pub depends_on: Vec<String>,
}

impl Task {
    pub fn split_tasks(tasks: &[Task]) -> (Vec<&Task>, Vec<&Task>) {
        let mut dependencies_tasks: HashSet<&str> = HashSet::new();
        tasks.iter().for_each(|task| {
            task.depends_on.iter().for_each(|dep_task| {
                dependencies_tasks.insert(dep_task);
            })
        });
        tasks.iter().partition(|task| {
            task.depends_on.is_empty()
                && !tasks
                    .iter()
                    .all(|_| dependencies_tasks.contains(task.name.as_str()))
        })
    }
}

#[cfg(test)]
mod helpers_tests {
    use super::{Graph, Task};
    use std::collections::HashMap;
    //
    //Test helpers
    type TestInputTask = (&'static str, &'static [&'static str]);
    impl Task {
        pub fn from_formatted(formatted_tasks: &[TestInputTask]) -> Vec<Task> {
            formatted_tasks.iter().map(|t| Task::from(*t)).collect()
        }
    }
    impl From<TestInputTask> for Task {
        fn from(value: TestInputTask) -> Self {
            Task {
                name: value.0.to_owned(),
                depends_on: value
                    .1
                    .iter()
                    .map(|refer| refer.to_string())
                    .collect::<Vec<String>>(),
            }
        }
    }

    #[test]
    fn test_split_tasks() {
        let input: &[TestInputTask] = &[("once", &[]), ("once_b", &["once"]), ("third_task", &[])];
        let task_vec: Vec<Task> = Task::from_formatted(input);

        assert_eq!(
            Task::split_tasks(&task_vec).0.first().unwrap(),
            &task_vec.get(2).unwrap()
        )
    }

    #[test]
    fn split_multiple_tasks() {
        let input: &[TestInputTask] = &[
            ("once", &[]),
            ("once_b", &["once"]),
            ("third_task", &[]),
            ("once_c", &["once", "once_b"]),
            ("speedy", &[]),
            ("err", &[]),
        ];

        let tasks: Vec<Task> = Task::from_formatted(input);
        let (indipendent_tasks, dependent_tasks) = Task::split_tasks(&tasks);
        assert_eq!(
            indipendent_tasks,
            &[
                tasks.get(2).unwrap(),
                tasks.get(4).unwrap(),
                tasks.get(5).unwrap()
            ]
        );
        assert_eq!(
            dependent_tasks,
            vec![
                tasks.first().unwrap(),
                tasks.get(1).unwrap(),
                tasks.get(3).unwrap()
            ]
        )
    }

    #[test]
    fn split_bigger_list() {
        let input: &[TestInputTask] = &[
            ("2.2_task", &["1.4_task"]),
            ("0.8_task", &[]),
            ("1.3_task", &["0.6_task"]),
            (
                "1.4_task",
                &["0.6_task", "0.7_task", "0.8_task", "0.9_task", "0.10_task"],
            ),
            ("0.5_task", &[]),
            ("0.3_task", &[]),
            ("0.2_task", &[]),
            ("1.1_task", &["0.2_task"]),
            ("0.7_task", &[]),
            ("0.6_task", &[]),
            ("0.11_task", &[]),
            (
                "1.2_task",
                &["0.2_task", "0.3_task", "0.4_task", "0.6_task", "0.10_task"],
            ),
            ("1.5_task", &["0.7_task"]),
            ("0.9_task", &[]),
            ("0.4_task", &[]),
            ("2.1_task", &["1.4_task", "1.5_task"]),
            ("0.10_task", &[]),
            ("0.1_task", &[]),
        ];
        let tasks = Task::from_formatted(input);

        let (indipendent, _) = Task::split_tasks(&tasks);
        [
            Task {
                name: "0.1_task".into(),
                depends_on: vec![],
            },
            Task {
                name: "0.5_task".into(),
                depends_on: vec![],
            },
            Task {
                name: "0.11_task".into(),
                depends_on: vec![],
            },
        ]
        .iter()
        .for_each(|el| assert!(indipendent.contains(&el)));
    }

    #[test]
    fn dep_list_to_nodes() {
        let one = Task {
            name: "one".to_owned(),
            depends_on: vec![],
        };

        let two = Task {
            name: "two".to_owned(),
            depends_on: vec!["one".to_owned()],
        };

        let three = Task {
            name: "three".to_owned(),
            depends_on: vec!["one".to_owned(), "two".to_owned()],
        };

        let dependent_dictionary: HashMap<String, usize> = HashMap::from([
            ("one".to_owned(), 1),
            ("two".to_owned(), 2),
            ("three".to_owned(), 3),
        ]);
        let dependencies_for_one =
            Graph::dependecies_lists_to_tuple_nodes(&one.depends_on, 1, &dependent_dictionary);
        assert_eq!(dependencies_for_one, None);

        let dependencies_for_two =
            Graph::dependecies_lists_to_tuple_nodes(&two.depends_on, 2, &dependent_dictionary);

        let dependencies_for_three =
            Graph::dependecies_lists_to_tuple_nodes(&three.depends_on, 3, &dependent_dictionary);

        assert_eq!(dependencies_for_two, Some(vec![(1, 2)]));
        assert_eq!(dependencies_for_three, Some(vec![(1, 3), (2, 3)]));
    }
}
