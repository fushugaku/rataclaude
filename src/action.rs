#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    ClaudeCode,
    FileBrowser,
}

#[derive(Debug, Clone)]
pub enum Action {
    Quit,
    ToggleFocus,
    FocusPane(FocusTarget),
    ResizePanes(i16),
    SwitchTab(ActiveTab),

    // PTY actions
    PtyInput(Vec<u8>),

    // Git navigation
    GitNavUp,
    GitNavDown,
    GitToggleStage,
    GitStageAll,
    GitShowDiff,
    GitDiscardFile,
    GitExpandFile,

    // Diff navigation
    DiffScrollUp,
    DiffScrollDown,
    DiffScrollAmount(i16),
    DiffScrollLeft,
    DiffScrollRight,
    DiffNextHunk,
    DiffPrevHunk,
    DiffClose,
    DiffToggleSelect,
    DiffSendLines,

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

    // File browser navigation
    FBNavUp,
    FBNavDown,
    FBEnter,
    FBParentDir,
    FBSwitchPanel,
    FBPageUp,
    FBPageDown,

    // File browser operations
    FBCopy,
    FBMove,
    FBDelete,
    FBRename,
    FBMkdir,

    // File browser misc
    FBToggleHidden,
    FBRefresh,
}

#[derive(Debug, Clone, Copy)]
pub enum FocusTarget {
    Pty,
    GitStatus,
    DiffView,
}
