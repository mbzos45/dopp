## ADDED Requirements

### Requirement: Initial window size based on container count
On initial load, the window size SHALL be set based on the current number of containers.

#### Scenario: Initial sizing
- **WHEN** the container list is first loaded
- **THEN** the window is resized to match the calculated size for the current container count

### Requirement: Expand window on increased container count
If the container count increases after refresh, the window size SHALL expand to fit the new count.

#### Scenario: Container count increases
- **WHEN** a refresh results in a higher container count than previously observed
- **THEN** the window size expands to fit the new count

### Requirement: Do not shrink window on decreased container count
If the container count decreases after refresh, the window size SHALL remain unchanged.

#### Scenario: Container count decreases
- **WHEN** a refresh results in a lower container count than previously observed
- **THEN** the window size remains unchanged
