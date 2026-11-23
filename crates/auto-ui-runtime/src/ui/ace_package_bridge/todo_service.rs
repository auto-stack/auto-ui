use ace_av::age_package::live_session::HostInput;
use ase_m0::todo_service::{
    Argument, Generation, Lease, RealmId, Reply, RequestId, ServiceBinding, ServiceError, Ticket,
    Todo, MAX_REQUEST_TIME, TODO_APP, TODO_SERVICE, TODO_VERSION,
};
use std::sync::Mutex;

pub(super) const REQUEST_FIELD: &str = "todo_request";
pub(super) const OPERATION_FIELD: &str = "todo_operation";
pub(super) const ID_FIELD: &str = "todo_id";
pub(super) const TEXT_FIELD: &str = "todo_text";
pub(super) const DONE_FIELD: &str = "todo_done";

pub(super) struct TodoTab {
    lease: Lease,
    pending: Option<Pending>,
    completed: i32,
}

struct Pending {
    sequence: i32,
    id: RequestId,
    ticket: Mutex<Ticket>,
    operation: Operation,
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    ReadBack { uncertain: bool },
    Create,
    Update,
    Flag,
}

impl TodoTab {
    pub(super) fn open(binding: &ServiceBinding, realm: RealmId) -> Result<Self, ServiceError> {
        let lease = binding.open(
            realm,
            &format!("app://{TODO_APP}/"),
            TODO_SERVICE,
            TODO_VERSION,
        )?;
        Ok(Self {
            lease,
            pending: None,
            completed: 0,
        })
    }

    /// Submit one authored request once. A UI frame may be observed repeatedly.
    pub(super) fn observe(
        &mut self,
        sequence: i32,
        operation: &str,
        id: i32,
        text: &str,
        done: bool,
    ) -> Option<HostInput> {
        if sequence <= self.completed || self.pending.is_some() {
            return None;
        }
        let (kind, arguments) = match operation {
            "list_todos" => (Operation::List, Vec::new()),
            "create_todo" => (Operation::Create, vec![Argument::Text(text.into())]),
            "update_todo" => (
                Operation::Update,
                vec![
                    Argument::Int(id),
                    Argument::Text(text.into()),
                    Argument::Bool(done),
                ],
            ),
            "delete_todo" => (Operation::Flag, vec![Argument::Int(id)]),
            "remove_completed" | "set_all_todos" => (Operation::Flag, vec![Argument::Bool(done)]),
            _ => return Some(self.complete(sequence, 1, None)),
        };
        match self.start(sequence, operation, kind, arguments, None) {
            Ok(()) => None,
            Err(_) => Some(self.complete(sequence, 1, None)),
        }
    }

    fn start(
        &mut self,
        sequence: i32,
        operation: &str,
        kind: Operation,
        arguments: Vec<Argument>,
        generation: Option<Generation>,
    ) -> Result<(), ServiceError> {
        let ticket = self
            .lease
            .submit(operation, arguments, generation, MAX_REQUEST_TIME)?;
        self.pending = Some(Pending {
            sequence,
            id: ticket.id(),
            ticket: Mutex::new(ticket),
            operation: kind,
        });
        Ok(())
    }

    pub(super) fn poll(&mut self) -> Option<HostInput> {
        let result = self.pending.as_mut()?.ticket.get_mut().ok()?.poll()?;
        let pending = self.pending.take().expect("polled lease owns its ticket");
        let sequence = pending.sequence;
        if result.id != pending.id {
            return Some(self.complete(sequence, 1, None));
        }
        let generation = result.id.generation;
        match (pending.operation, result.result) {
            (Operation::List, Ok(Reply::List(rows))) => {
                Some(self.complete(sequence, 0, Some(rows)))
            }
            (Operation::ReadBack { uncertain }, Ok(Reply::List(rows))) => {
                Some(self.complete(sequence, if uncertain { 2 } else { 0 }, Some(rows)))
            }
            (Operation::List, _) => Some(self.complete(sequence, 1, None)),
            (Operation::ReadBack { .. }, _) => Some(self.complete(sequence, 2, None)),
            (Operation::Create, Ok(Reply::Created(_)))
            | (Operation::Update, Ok(Reply::Updated(Some(_))))
            | (Operation::Flag, Ok(Reply::Flag(true))) => self.refresh(sequence, generation, false),
            (
                _,
                Err(
                    ServiceError::TimedOut {
                        may_have_committed: true,
                    }
                    | ServiceError::Cancelled {
                        may_have_committed: true,
                    },
                ),
            ) => self.refresh(sequence, generation, true),
            _ => Some(self.complete(sequence, 1, None)),
        }
    }

    fn refresh(
        &mut self,
        sequence: i32,
        generation: Generation,
        uncertain: bool,
    ) -> Option<HostInput> {
        let kind = Operation::ReadBack { uncertain };
        match self.start(sequence, "list_todos", kind, Vec::new(), Some(generation)) {
            Ok(()) => None,
            Err(_) => Some(self.complete(sequence, 2, None)),
        }
    }

    fn complete(&mut self, sequence: i32, status: i32, rows: Option<Vec<Todo>>) -> HostInput {
        self.completed = sequence;
        let ok = rows.is_some();
        let rows = rows
            .unwrap_or_default()
            .into_iter()
            .map(|todo| {
                HostInput::Record(vec![
                    ("id".into(), HostInput::I32(todo.id)),
                    ("text".into(), HostInput::Text(todo.text)),
                    ("done".into(), HostInput::Bool(todo.done)),
                ])
            })
            .collect();
        HostInput::Record(vec![
            ("request".into(), HostInput::I32(sequence)),
            ("ok".into(), HostInput::Bool(ok)),
            ("status".into(), HostInput::I32(status)),
            ("rows".into(), HostInput::List(rows)),
        ])
    }
}
