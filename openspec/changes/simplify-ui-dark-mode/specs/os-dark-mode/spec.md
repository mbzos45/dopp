## ADDED Requirements

### Requirement: Automatic theme selection
The UI SHALL automatically switch between light and dark themes based on the OS color scheme preference.

#### Scenario: OS preference changes to dark
- **WHEN** the OS reports a dark color scheme preference
- **THEN** the UI applies the dark theme

#### Scenario: OS preference changes to light
- **WHEN** the OS reports a light color scheme preference
- **THEN** the UI applies the light theme
