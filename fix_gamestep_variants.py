import os

path = r'c:\proiecte-vue\monopoly-world-edition\backend\game-engine\src\game.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Replace WaitingForAirportDecision with WaitingForAirportDecision { .. } where appropriate
# We need to be careful not to touch the definition and legitimate struct usage
# But in match arms it's usually just the name

# Match arms and equality checks
text = text.replace('GameStep::WaitingForAirportDecision |', 'GameStep::WaitingForAirportDecision { .. } |')
text = text.replace('GameStep::WaitingForAirportDecision =>', 'GameStep::WaitingForAirportDecision { .. } =>')
text = text.replace('GameStep::WaitingForAirportDecision {', 'GameStep::WaitingForAirportDecision_TEMP_{') # preserve definition

# Simple WaitingForAirportDecision not followed by {
import re
text = re.sub(r'GameStep::WaitingForAirportDecision(?![^{])', 'GameStep::WaitingForAirportDecision { .. }', text)

# Restore definition
text = text.replace('GameStep::WaitingForAirportDecision_TEMP_{', 'GameStep::WaitingForAirportDecision {')

# Repeat for Destination
text = text.replace('GameStep::WaitingForAirportDestination |', 'GameStep::WaitingForAirportDestination { .. } |')
text = text.replace('GameStep::WaitingForAirportDestination =>', 'GameStep::WaitingForAirportDestination { .. } =>')
text = re.sub(r'GameStep::WaitingForAirportDestination(?![^{])', 'GameStep::WaitingForAirportDestination { .. }', text)

# Fix double { .. } { .. } if it happened
text = text.replace('{ .. } { .. }', '{ .. }')

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
