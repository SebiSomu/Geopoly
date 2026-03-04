# GEOPOLY: World Edition Online

> **Disclaimer:** This project is a fan-made, non-commercial digital adaptation inspired by the "Monopoly" board game franchise. All trademarks, names, and logos are the property of their respective owners (Hasbro). This software is intended for educational and personal entertainment purposes only and is not affiliated with or endorsed by the original copyright holders.

Geopoly is a modern, fast-paced digital board game inspired by the classic Monopoly "Here & Now: World Edition". Travel the world, collect colorful stamps for your passport, and outsmart your opponents in a neon-infused global race for dominance.

## Project Overview

Geopoly features a complete passport stamp collection system, dynamic board logic, and real-time interaction between players. The game is designed for a multi-client experience, allowing multiple players to connect to a central server and play together across different devices or browsers, provided they can reach the server's network address.

## Key Features

- **Full Passport System**: Purchase destinations and collect unique stamps.
- **Multi-Client Support**: Play with friends by connecting multiple clients to the same backend server.
- **Dynamic Board Logic**: Powered by a high-performance engine written in Rust.
- **Real-time Interaction**: Instant updates and fluid animations.
- **"Here & Now" & "Chance" Cards**: Strategic card system including Just Say No!, Intercept, and Forced Deals.
- **Premium Interface**: Modern dark-mode UI with vibrant cyan and yellow neon aesthetics.

## Technology Stack

The project is split into two specialized parts for maximum performance and a modern web experience:

### Frontend (User Interface)
- **Vue 3 (Composition API)**: For a reactive and component-based user interface.
- **TypeScript**: Ensuring type safety and robust code architecture.
- **Vite**: Ultra-fast build tool and development server.
- **Apollo Client / GraphQL**: Modern data fetching and state management.
- **Vanilla CSS**: Custom-crafted styles featuring glassmorphism and neon effects.

### Backend (Game Engine & Server)
- **Rust**: A high-performance, memory-safe language handling all game rules and state.
- **Juniper / GraphQL**: Efficient communication between the frontend and game engine.
- **Tokio**: Async runtime for handling multiple concurrent game sessions and clients.
- **MongoDB**: Secure data persistence that stores the users and the extensive game action history, which is fetched via GraphQL and evaluated contextually in real-time for every action.

## How to Run

### Prerequisites
- Node.js (v16+)
- Rust & Cargo (latest stable)

### 1. Start the Backend (Rust)
Navigate to the backend directory and run the server:
```bash
cd backend/server
cargo run
```

### 2. Start the Frontend (Vue)
Navigate to the frontend directory, install dependencies, and start the development server:
```bash
cd frontend
npm install
npm run dev
```

The game defaults to `http://localhost:5173`. For multi-client play, ensure the clients can reach the server's IP address on the network.

## Game Rules Summary

- **Starting State**: Every player begins with G1500 and 2 "Here & Now" cards.
- **Movement**: Roll two dice. Doubles grant an extra turn (3 doubles in a row sends the player to Jail).
- **Forced Deal**: Landing on the Business icon triggers a deal where you can swap stamps with opponents.
- **Winning**: Fill your passport completely or be the last player with capital.

## Security Note

This project uses bcrypt for secure password hashing in the backend. Environment variables and sensitive configuration files are listed in .gitignore to prevent exposure in public repositories.

## Author
Sebi Somu
