# Preserve editor drafts during canvas and error updates

Canvas undo, redo and cancellation change geometry independently of model edits. Their repaints now retain unrelated creation, rename, scalar and import drafts plus open disclosures. Rejected operations also retain values so users can correct and retry. Returned focus opens its disclosure ancestors; successful model actions retain their existing draft-reset and focus semantics.

The published UI reproduced canvas undo replacing an unsaved rename with its committed value. Fresh browser assertions now cover undo/redo/cancel, rejected creation, preserved independent rename, disclosure state and unchanged committed model. The persistent live region remains connected. Error-path caret retention beyond existing focus behavior is not claimed.

Offline native checks and release WASM pass with 41 tests. Two final browser captures verify the walkthrough and focused regression; screenshots and complete raw artifacts stay in the external audit directory. Depends on F36 #51; land that prerequisite first, retarget to integration and revalidate. Never merge into its feature branch.
