#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    ToggleFocus,
    FocusPane(FocusTarget),
    ResizePanes(i16),

    // PTY actions
    PtyInput(Vec<u8>),

    // Git navigation
    GitNavUp,
    GitNavDown,
    GitToggleStage,
    GitStageAll,
    GitShowDiff,
    GitDiscardFile,

    // Diff navigation
    DiffScrollUp,
    DiffScrollDown,
    DiffScrollAmount(i16),
    DiffNextHunk,
    DiffPrevHunk,
    DiffClose,

    // Send to Claude
    SendToClaude,
    SendToClaudeWithPrompt,
    ToggleMultiSelect,

    // Git operations
    Commit,
    CommitAndPush,
    Push,
    Pull,
    CreateBranch,
    CheckoutBranch(String),
    BranchList,
    Stash,
    StashPop,
}

#[derive(Debug, Clone, Copy)]
pub enum FocusTarget {
    Pty,
    GitStatus,
    DiffView,
}
