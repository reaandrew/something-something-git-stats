use std::collections::HashMap;
use bytesize::ByteSize;
use chrono::{Datelike, Timelike};
use crate::{GitCommit, GitStat, GitStatsViewModel, LineStats};
use crate::duplicates::DuplicateDetector;
use crate::stats::{FileStats, MessageStats, PunchStats, SummaryStats};
use crate::viewmodel::{FilesValue, KeyValue, LinesValue, PunchesValue, SummaryViewModel};

struct SummaryStatsCollector {
    pub(crate) summary: SummaryStats,
}

impl SummaryStatsCollector{
    fn default() -> Self{
        Self{
            summary: Default::default()
        }
    }
}

impl GitStat for SummaryStatsCollector {
    fn process(&mut self, commit: &GitCommit) {
        self.summary.commit_count += 1;

        if self.summary.date_first_commit.is_empty() {
            self.summary.date_first_commit = commit.date.to_string();
        }

        if self.summary.first_committer.is_empty() {
            self.summary.first_committer = String::from(&commit.author);
        }

        self.summary.total_lines_added += commit.total_lines_added();
        self.summary.total_lines_deleted += commit.total_lines_deleted();
    }

    fn update(&self, viewmodel: &mut GitStatsViewModel) {
        viewmodel.summary.push(SummaryViewModel {
            name: "First committer".to_string(),
            value: self.summary.first_committer.clone()
        });
        viewmodel.summary.push(SummaryViewModel {
            name: "Date of first commit".to_string(),
            value: self.summary.date_first_commit.clone()
        });
        viewmodel.summary.push(SummaryViewModel {
            name: "Number of commits_collection".to_string(),
            value: self.summary.commit_count.to_string()
        });
        viewmodel.summary.push(SummaryViewModel {
            name: "Total lines_collection added".to_string(),
            value: self.summary.total_lines_added.to_string()
        });
        viewmodel.summary.push(SummaryViewModel {
            name: "Total lines_collection deleted".to_string(),
            value: self.summary.total_lines_deleted.to_string()
        });



    }
}

struct TotalCommitsByDayCollector {
    total_commits_by_day: HashMap<String, i32>
}

impl TotalCommitsByDayCollector{
    fn default() -> Self{
        Self{
            total_commits_by_day: Default::default()
        }
    }
}

impl GitStat for TotalCommitsByDayCollector {
    fn process(&mut self, commit: &GitCommit) {
        let stat = self.total_commits_by_day.entry(commit.day_key())
            .or_insert(0);
        *stat += 1;
    }

    fn update(&self, viewmodel: &mut GitStatsViewModel) {
        for (key, value) in self.total_commits_by_day.clone() {
            viewmodel.total_commits_by_day.push(KeyValue {
                key,
                value
            })
        }
        viewmodel.total_commits_by_day.sort_by(|a, b| a.key.cmp(&b.key));
    }
}

struct TotalLinesByDayCollector {
    total_lines_by_day: HashMap<String, LineStats>
}

impl TotalLinesByDayCollector{
    fn default() -> Self{
        Self{
            total_lines_by_day: Default::default()
        }
    }
}

impl GitStat for TotalLinesByDayCollector {
    fn process(&mut self, commit: &GitCommit) {
        let stat = self.total_lines_by_day.entry(commit.day_key())
            .or_insert(LineStats {
                added: 0,
                deleted: 0,
            });
        stat.added += commit.total_lines_added();
        stat.deleted += commit.total_lines_deleted();
    }

    fn update(&self, viewmodel: &mut GitStatsViewModel) {
        for (key, value) in self.total_lines_by_day.clone(){
            viewmodel.total_lines_by_day.push(LinesValue{
                key,
                lines_added: value.added,
                lines_deleted: value.deleted
            })
        }
        viewmodel.total_lines_by_day.sort_by(|a, b| a.key.cmp(&b.key));
    }
}

struct TotalFilesByDayCollector{
    total_files_by_day: HashMap<String, FileStats>
}

impl TotalFilesByDayCollector{
    fn default() -> Self{
        Self{
            total_files_by_day: Default::default()
        }
    }
}

impl GitStat for TotalFilesByDayCollector{
    fn process(&mut self, commit: &GitCommit) {
        let stat = self.total_files_by_day.entry(commit.day_key())
            .or_insert(FileStats {
                added: 0,
                modified: 0,
                deleted: 0,
                renamed: 0
            });
        stat.added += commit.total_files_added();
        stat.deleted += commit.total_files_deleted();
        stat.modified += commit.total_files_modified();
        stat.renamed += commit.total_files_renamed();
    }

    fn update(&self, viewmodel: &mut GitStatsViewModel) {
        for (key, value) in self.total_files_by_day.clone(){
            viewmodel.total_files_by_day.push(FilesValue{
                key,
                files_added: value.added,
                files_deleted: value.deleted,
                files_modified: value.modified,
                files_renamed: value.renamed
            })
        }
        viewmodel.total_files_by_day.sort_by(|a, b| a.key.cmp(&b.key));
    }
}

struct MessageStatsCollector {
    count: i32,
    total_message_lines: i32,
    total_message_size: i32,
    message_stats: MessageStats,
}


impl MessageStatsCollector{
    fn default() -> Self{
        Self{
            count: 0,
            total_message_lines: 0,
            total_message_size: 0,
            message_stats: Default::default()
        }
    }
}

impl GitStat for MessageStatsCollector {
    fn process(&mut self, commit: &GitCommit) {
        self.count += 1;
        self.total_message_lines += commit.total_message_lines();
        self.total_message_size += commit.total_message_size();

        if commit.total_message_size() > self.message_stats.max_size {
            self.message_stats.max_size = commit.total_message_size()
        }

        if commit.total_message_lines() > self.message_stats.max_lines {
            self.message_stats.max_lines = commit.total_message_lines();
        }

        if commit.total_message_size() <= self.message_stats.min_size {
            self.message_stats.min_size = commit.total_message_size()
        }

        if commit.total_message_lines() <= self.message_stats.min_lines {
            self.message_stats.min_lines = commit.total_message_lines();
        }

        self.message_stats.avg_size = self.total_message_size / self.count;
        self.message_stats.avg_lines = self.total_message_lines / self.count;
    }

    fn update(&self, viewmodel: &mut GitStatsViewModel) {
        viewmodel.summary.push(SummaryViewModel{
            name: "Total size of all commit messages".to_string(),
            value: ByteSize(self.total_message_size as u64).to_string()
        });

        viewmodel.summary.push(SummaryViewModel{
            name: "Total number of lines_collection across all commit messages".to_string(),
            value: self.total_message_lines.to_string()
        });

        viewmodel.summary.push(SummaryViewModel{
            name: "Max number of lines_collection in a commit message".to_string(),
            value: self.message_stats.max_lines.to_string()
        });

        viewmodel.summary.push(SummaryViewModel{
            name: "Max size of a commit message".to_string(),
            value: ByteSize(self.message_stats.max_size as u64).to_string()
        });

        viewmodel.summary.push(SummaryViewModel{
            name: "Avg number of lines_collection in a commit message".to_string(),
            value: self.message_stats.avg_lines.to_string()
        });

        viewmodel.summary.push(SummaryViewModel{
            name: "Avg size of a commit message".to_string(),
            value: ByteSize(self.message_stats.avg_size as u64).to_string()
        });

    }
}

struct SimilarFilesChangingCollector{
    dup_detector: DuplicateDetector,
}

impl SimilarFilesChangingCollector{
    fn _default() -> Self{
        Self{
            dup_detector: DuplicateDetector::new(10)
        }
    }
}

impl GitStat for SimilarFilesChangingCollector{
    fn process(&mut self, commit: &GitCommit) {
        let files = commit.file_operations
            .iter()
            .map(|x|x.file.as_str())
            .collect::<Vec<&str>>();
        self.dup_detector.add(files);
    }

    fn update(&self, _viewmodel: &mut GitStatsViewModel) {
        // TODO: implement me
    }
}

struct PunchCardCollector{
    punchcard: HashMap<String, PunchStats>
}

impl PunchCardCollector{
    fn default() -> Self{
        Self{
            punchcard: HashMap::new()
        }
    }
}

impl GitStat for PunchCardCollector{
    fn process(&mut self, commit: &GitCommit) {
        let stat = self.punchcard.entry(commit.hour_key_by_weekday())
            .or_insert(PunchStats {
                weekday: commit.date.weekday().num_days_from_sunday(),
                hour: commit.date.hour(),
                commits: 0
            });
        stat.commits += 1
    }

    fn update(&self, viewmodel: &mut GitStatsViewModel) {
        for (_, value) in self.punchcard.clone(){
            viewmodel.punch_data.push(PunchesValue{
                weekday: value.weekday,
                hour: value.hour,
                commits: value.commits
            })
        }
    }
}

pub fn create_stat_collectors() -> Vec<Box<dyn GitStat>> {
    let stats_functions: Vec<Box<dyn GitStat>> = vec![
        Box::new(SummaryStatsCollector::default()),
        Box::new(TotalCommitsByDayCollector::default()),
        Box::new(TotalLinesByDayCollector::default()),
        Box::new(MessageStatsCollector::default()),
        //Box::new(SimilarFilesChangingCollector::default()),
        Box::new(TotalFilesByDayCollector::default()),
        Box::new(PunchCardCollector::default())
    ];
    stats_functions
}


#[cfg(test)]
mod summary_stats_collector_tests {
    use chrono::{DateTime, Duration, Utc};
    use crate::{GitCommit, GitStat, GitStats};
    use crate::collectors::SummaryStatsCollector;
    use crate::models::LineStat;
    use crate::process::process_commit;

    #[test]
    fn test_overall_commit_count_with_1_commit() {
        let mut commit: GitCommit = GitCommit::default();
        commit.commit_hash = String::from("123");

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit, &stat_functions, &mut stats, &||{});

        assert_eq!(1, stats.summary.commit_count);
    }

    #[test]
    fn test_date_1_commit(){
        let mut commit: GitCommit = GitCommit::default();
        commit.date = DateTime::from(Utc::now());

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit, &stat_functions, &mut stats, &||{});

        assert_eq!(commit.date.to_string(), stats.summary.date_first_commit);
    }

    #[test]
    fn test_date_2_commits(){
        let mut commit_1: GitCommit = GitCommit::default();
        commit_1.date = DateTime::from(Utc::now() - Duration::days(2));
        let mut commit_2: GitCommit = GitCommit::default();
        commit_2.date = DateTime::from(Utc::now());

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit_1, &stat_functions, &mut stats, &||{});
        process_commit(&commit_2, &stat_functions, &mut stats, &||{});

        assert_eq!(commit_1.date.to_string(), stats.summary.date_first_commit);
    }

    #[test]
    fn test_first_committer_1_commit(){
        let mut commit: GitCommit = GitCommit::default();
        commit.author = String::from("Bob");

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit, &stat_functions, &mut stats, &||{});

        assert_eq!(stats.summary.first_committer, "Bob");
    }

    #[test]
    fn test_first_committer_2_commits(){
        let mut commit_1: GitCommit = GitCommit::default();
        commit_1.author = String::from("Jeff");
        let mut commit_2: GitCommit = GitCommit::default();
        commit_2.author = String::from("Alan");

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit_1, &stat_functions, &mut stats, &||{});
        process_commit(&commit_2, &stat_functions, &mut stats, &||{});

        assert_eq!(stats.summary.first_committer, "Jeff");
    }

    #[test]
    fn test_lines_added_1_commit(){
        let mut commit: GitCommit = GitCommit::default();
        commit.line_stats = vec![LineStat{
            lines_added: 10,
            lines_deleted: 0
        }];

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit, &stat_functions, &mut stats, &||{});

        assert_eq!(stats.summary.total_lines_added, 10);
    }

    #[test]
    fn test_lines_added_2_commit(){
        let mut commit_1: GitCommit = GitCommit::default();
        commit_1.line_stats = vec![LineStat{
            lines_added: 10,
            lines_deleted: 0
        }];

        let mut commit_2: GitCommit = GitCommit::default();
        commit_2.line_stats = vec![LineStat{
            lines_added: 5,
            lines_deleted: 0
        }];

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit_1, &stat_functions, &mut stats, &||{});
        process_commit(&commit_2, &stat_functions, &mut stats, &||{});


        assert_eq!(stats.summary.total_lines_added, 15);
    }

    #[test]
    fn test_lines_deleted_1_commit(){
        let mut commit: GitCommit = GitCommit::default();
        commit.line_stats = vec![LineStat{
            lines_added: 0,
            lines_deleted: 2
        }];

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit, &stat_functions, &mut stats, &||{});

        assert_eq!(stats.summary.total_lines_deleted, 2);
    }

    #[test]
    fn test_lines_deleted_2_commit(){
        let mut commit_1: GitCommit = GitCommit::default();
        commit_1.line_stats = vec![LineStat{
            lines_added: 0,
            lines_deleted: 2
        }];

        let mut commit_2: GitCommit = GitCommit::default();
        commit_2.line_stats = vec![LineStat{
            lines_added: 0,
            lines_deleted: 7
        }];

        let stat_functions: Vec<Box<dyn GitStat>> = vec![
            Box::new(SummaryStatsCollector {})
        ];

        let mut stats: GitStats = Default::default();

        process_commit(&commit_1, &stat_functions, &mut stats, &||{});
        process_commit(&commit_2, &stat_functions, &mut stats, &||{});


        assert_eq!(stats.summary.total_lines_deleted, 9 );
    }
}