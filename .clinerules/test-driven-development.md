# Test-Driven Development (TDD)

## Core Principle
**Write tests BEFORE writing implementation code.** Follow the Red-Green-Refactor cycle strictly.

## TDD Cycle

### 1. RED: Write a Failing Test
- Before implementing any feature, write a test that defines the expected behavior
- Run the test and confirm it fails (for the right reason)
- The test should fail because the functionality doesn't exist yet

### 2. GREEN: Write Minimal Code to Pass
- Write the simplest code that makes the test pass
- Don't add extra features or "future-proof" the code
- Run the test and confirm it passes

### 3. REFACTOR: Improve Code Quality
- Clean up the code while keeping tests passing
- Remove duplication, improve naming, enhance structure
- Run tests after each refactor to ensure nothing broke

### 4. REPEAT
- Move to the next test case and start the cycle again

## When to Write Tests

**Always write tests for:**
- Public APIs and interfaces
- Data validation logic
- Configuration loading
- Data transformations (parsing, mapping, formatting)
- Error handling paths
- Integration points (Notion API, Anki generation)

**Tests can be skipped for:**
- Simple one-line getters/setters with no logic
- Third-party library wrappers (test integration, not the library itself)
- Temporary debugging code

## Naming Conventions
- Test files: `test_<module_name>.py`
- Test functions: `test_<function_name>_<scenario>()`
- Test classes: `Test<ClassName>`