// SPDX-License-Identifier: EPL-2.0
//! Existing-transaction bridge for attachment before initializer queries.
use super::*;
use syson_model::Transaction;

/// Own the already staged transaction: refusal/cancellation drops all staged
/// creation/default changes. Success returns that same transaction for one
/// caller commit; Editor identity/revision/failed checks stay authoritative.
/// A caller-supplied plan is not certified as generated from this snapshot.
pub fn apply_staged_plan(mut transaction:Transaction,root:&str,plan:&Plan,cancelled:bool)->Result<Transaction,Error> {
    if cancelled { return Err(Error::Cancelled) }
    if Index::new(transaction.document())?.node(root)?.classifier!=plan.classifier { return Err(Error::ClassMismatch) }
    apply_scalar_steps(&mut transaction,root,&plan.steps)?;
    Ok(transaction)
}
/// Attach the validated preallocated subtree through the model transaction
/// first, then call this on that owned transaction. Queries see its staged
/// document, including the new root. Defaults and attachment share one commit.
/// No raw allocation, projection, inverse notifications or reference mutation.
pub fn initialize_staged(transaction:Transaction,root:&str,encoding:ReferenceEncoding,cancelled:bool)->Result<Transaction,Error> {
    if cancelled { return Err(Error::Cancelled) }
    let class=Index::new(transaction.document())?.node(root)?.classifier;
    let mut context=NativeContext::new(transaction.document(),root)?;
    context.reference_encoding=encoding;
    let planned=match syson_initializer::plan(class,&mut context) {
        Outcome::Planned(plan)=>plan,
        Outcome::Incomplete(incomplete)=>return Err(Error::Planning(incomplete.problem)),
    };
    apply_staged_plan(transaction,root,&planned,false)
}
