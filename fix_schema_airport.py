import os

path = r'c:\proiecte-vue\monopoly-world-edition\backend\server\src\schema.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Fix airport decision validation
old_decision = """            // Validate turn
            let current_idx = game.current_player_idx;
            if game.players[current_idx].name != username {
                return Err(Error::new("Nu este randul tau!"));
            }"""

new_decision = """            // Validate turn - support out-of-turn movement
            let buyer_idx = if let GameStep::WaitingForAirportDecision { buyer_idx } = game.step {
                buyer_idx
            } else {
                game.current_player_idx
            };

            if game.players[buyer_idx].name != username {
                return Err(Error::new("Nu este rândul tău să alegi zborul!"));
            }"""

# Fix airport destination validation
old_dest = """            // Validate turn
            let current_idx = game.current_player_idx;
            if game.players[current_idx].name != username {
                return Err(Error::new("Nu este randul tau!"));
            }"""

new_dest = """            // Validate turn - support out-of-turn movement
            let buyer_idx = if let GameStep::WaitingForAirportDestination { buyer_idx } = game.step {
                buyer_idx
            } else {
                game.current_player_idx
            };

            if game.players[buyer_idx].name != username {
                return Err(Error::new("Nu este rândul tău să alegi destinația!"));
            }"""

# Replace resolve_airport_decision (first occurrence after its function def)
# Resolve Airport flight decision is around line 603
func_decision = 'async fn resolve_airport_decision'
idx_decision = text.find(func_decision)
if idx_decision != -1:
    text_part = text[idx_decision:]
    text_part = text_part.replace(old_decision, new_decision, 1)
    text = text[:idx_decision] + text_part

# Replace resolve_airport_destination
func_dest = 'async fn resolve_airport_destination'
idx_dest = text.find(func_dest)
if idx_dest != -1:
    text_part = text[idx_dest:]
    text_part = text_part.replace(old_dest, new_dest, 1)
    text = text[:idx_dest] + text_part

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
