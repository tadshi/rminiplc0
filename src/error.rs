#[derive(Debug)]
pub struct CompilationError {
    pos: (usize, usize),
    err_code: ErrorCode,
}

impl CompilationError {
    pub fn new(line: usize, col: usize, err: ErrorCode) -> CompilationError {
        CompilationError {
            pos: (line, col),
            err_code: err,
        }
    }

    pub fn new_packed(ptr: (usize, usize), err: ErrorCode) -> CompilationError {
        CompilationError {
            pos: ptr,
            err_code: err,
        }
    }

    pub fn get_err_code(&self) -> &ErrorCode {
        &self.err_code
    }
}

#[derive(PartialEq, Debug)]
pub enum ErrorCode {
    ErrNoError, // Should be only used internally.
    ErrStreamError,
    ErrEOF,
    ErrInvalidInput,
    ErrInvalidIdentifier,
    ErrIntegerOverflow, // int32_t overflow.
    ErrNoBegin,
    ErrNoEnd,
    ErrNeedIdentifier,
    ErrConstantNeedValue,
    ErrNoSemicolon,
    ErrInvalidVariableDeclaration,
    ErrIncompleteExpression,
    ErrNotDeclared,
    ErrAssignToConstant,
    ErrDuplicateDeclaration,
    ErrNotInitialized,
    ErrInvalidAssignment,
    ErrInvalidPrint,
}
