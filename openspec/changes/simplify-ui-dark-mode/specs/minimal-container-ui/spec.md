## ADDED Requirements

### Requirement: Minimal container row layout
The UI SHALL display exactly one container per row, showing only the container name and the operation buttons.

#### Scenario: Container row content
- **WHEN** the container list is rendered
- **THEN** each row shows the container name and the available operation buttons only

### Requirement: No extra headers or decorative sections
The UI SHALL NOT display additional headers or decorative sections beyond the refresh control.

#### Scenario: Header visibility
- **WHEN** the main UI is shown
- **THEN** no extra header or decorative sections are visible besides the refresh control
