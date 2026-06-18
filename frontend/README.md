# GOBLIN DNS Redirector - Frontend

A React-based web UI for managing and monitoring the GOBLIN DNS redirector, a security research tool for intercepting and overriding DNS resolutions.

## Tech Stack

- **Framework**: React 19 + TypeScript 6
- **Build Tool**: Vite 8
- **Routing**: React Router DOM 7
- **Data Fetching & Caching**: TanStack React Query 5
- **HTTP Client**: Axios
- **Runtime**: Node.js (ESM modules)

## Architecture Overview

```
                    ┌─────────────────────────────────────────────┐
                    │                   App.tsx                    │
                    │  ┌───────────────────────────────────────┐  │
                    │  │         QueryClientProvider            │  │
                    │  │         (TanStack React Query)         │  │
                    │  └───────────────────────────────────────┘  │
                    │  ┌───────────────────────────────────────┐  │
                    │  │           BrowserRouter                │  │
                    │  │                                       │  │
                    │  │  ┌─────────┐ ┌────────────┐          │  │
                    │  │  │  Nav    │ │   Footer   │          │  │
                    │  │  │  Bar   │ │            │          │  │
                    │  │  └─────────┘ └────────────┘          │  │
                    │  │                                       │  │
                    │  │  ┌────────────────────────────────┐  │  │
                    │  │  │           Routes               │  │  │
                    │  │  │         (Main Content)         │  │  │
                    │  │  └────────────────────────────────┘  │  │
                    │  └───────────────────────────────────────┘  │
                    └────────┬────────┬──────────────────┬────────┘
                             │        │                  │
                    ┌────────▼──┐ ┌───▼────────┐ ┌──────▼───────┐
                    │ Stats     │ │ Resolvers  │ │ Rules        │
                    │ Dashboard │ │            │ │              │
                    │           │ │ Compo-     │ │ Component    │
                    │ Component │ │ nents      │ │              │
                    └─────┬─────┘ └─────┬──────┘ └──────┬───────┘
                          │             │                │
                    ┌─────▼─────┐ ┌─────▼──────┐ ┌──────▼───────┐
                    │ useStats  │ │ useResolver │ │ useRules     │
                    │ Hooks     │ │ s          │ │ Hooks        │
                    └─────┬─────┘ └────────────┘ └──────┬───────┘
                          │                             │
                    ┌─────▼─────────────────────────────▼───────┐
                    │              api/client.ts                │
                    │  (Axios-based API layer)                  │
                    └─────────────────────┬─────────────────────┘
                                          │
                                          ▼
                               ┌───────────────────────┐
                               │    Backend API /api/v1 │
                               │    (nginx reverse      │
                               │     proxy)             │
                               └───────────────────────┘
```

## Project Structure

```
frontend/
├── public/                      # Static assets (favicon, etc.)
├── src/
│   ├── api/
│   │   └── client.ts            # Axios-based API layer
│   ├── components/
│   │   ├── Statistics/
│   │   │   └── StatsDashboard.tsx   # Dashboard view: stats summary + recent activity
│   │   ├── UpstreamResolvers/
│   │   │   └── ResolverList.tsx     # Resolver CRUD view
│   │   └── PriorityRules/
│   │       └── RuleList.tsx         # Rules CRUD view with inline form
│   ├── hooks/
│   │   ├── useResolvers.ts        # Resolver queries & mutations
│   │   ├── useRules.ts            # Rule queries & mutations
│   │   └── useStats.ts            # Stats queries
│   ├── types/
│   │   └── index.ts               # Shared TypeScript interfaces
│   ├── App.css                    # Global styles
│   ├── App.tsx                    # App shell (nav, routes, providers)
│   └── main.tsx                   # Entry point
├── Containerfile                  # Container build definition
├── docker-entrypoint.sh           # Container startup script
├── nginx.conf                     # Reverse proxy config (Served by nginx)
├── vite.config.ts                 # Vite build config with proxy settings
├── index.html                     # HTML entry point
└── package.json
```

## Components

### App (`App.tsx`)

The root component orchestrates the application shell, containing three primary concerns:

1. **Provider wrapping** — `QueryClientProvider` from TanStack React Query, configured with `refetchOnWindowFocus: false` and a single retry.
2. **Routing** — `BrowserRouter` with three named routes mapped to `Routes`:
   - `<App />` → StatsDashboard (`/`)
   - `<ResolverList />` → Resolvers (`/resolvers`)
   - `<RuleList />` → Priority Rules (`/rules`)
3. **Navigation** — A top `<nav>` bar with `NavLink` components highlighting the active route.

### StatsDashboard (`components/Statistics/StatsDashboard.tsx`)

Displays real-time statistics:

- Four summary stat cards (Total Hits, Unique Clients, Active Rules, Total Resolvers)
- A "Recent Activity" table of the latest 20 statistics entries
- Polls summary data every 10 seconds via `useStatsSummary`'s `refetchInterval`

### ResolverList (`components/UpstreamResolvers/ResolverList.tsx`)

Manages upstream DNS resolvers with:

- A list table showing resolver ID, IP address, and domain name
- An inline form toggled via state (`showForm`) for creating new resolver entries
- Delete actions with browser confirmation dialogs

### RuleList (`components/PriorityRules/RuleList.tsx`)

Manages DNS priority rules with:

- A table displaying rule ID, domain, client filter, target IP, and active status
- Inline form for creating rules (domain, client filter, target IP, active toggle)
- Actions for toggling rule active/inactive status and deletion

## Data Flow

```
Component (e.g. RuleList)
  │
  ▼
Custom Hook (e.g. useRules)
  │  └─ useQuery / useMutation (TanStack React Query)
  │
  ▼
API Client (api/client.ts)
  │  └─ Axios with interceptors
  │
  ▼
Backend (nginx reverse proxy → /api/v1/*)
```

### Layer 1: API Client (`api/client.ts`)

A thin Axios-based wrapper that exposes RESTful methods per resource domain:

- `resolversApi` — `getAll`, `getById`, `create`, `update`, `delete`
- `rulesApi` — `getAll`, `getById`, `create`, `update`, `toggle`, `delete`
- `statsApi` — `getAll`, `getByRule`, `getSummary`
- `healthApi` — `check`

All requests use a shared `baseURL` of `/api/v1` and proxy through nginx to the backend.

### Layer 2: Custom Hooks

Each feature has a set of hooks built on TanStack React Query:

- **Query hooks** — Wrap `useQuery` for read operations (e.g., `useRules`, `useStats`). Query keys follow the pattern `['resource']` or `['resource', specific_id]`.
- **Mutation hooks** — Wrap `useMutation` for create/update/delete/toggle operations (e.g., `useCreateRule`, `useUpdateResolver`). On success, invalidates the relevant query caches to trigger refetches.

Mutation invalidation strategy:

- Modifying resources invalidates both their own list (e.g., `['rules']`) and dependent caches like `['stats']` where applicable.

### Layer 3: TypeScript Types (`types/index.ts`)

Shared interfaces covering all domain models and API contracts:

| Domain Model | Request Types | API Types |
|---|---|---|
| `UpstreamResolver` | `CreateResolverRequest`, `UpdateResolverRequest` | — |
| `PriorityRule` | `CreateRuleRequest`, `UpdateRuleRequest` | — |
| `Statistic`, `StatsSummary` | — | — |
| — | — | `ApiResponse<T>`, `CreateResponse`, `UpdateResponse`, `DeleteResponse` |

## Deployment

### Container Build

The app is built and served via nginx:

```bash
podman build -t goblin-dns-frontend .
```

The `Containerfile` uses a multi-stage build: Vite builds the static assets, which are then served by an nginx image. `docker-entrypoint.sh` handles startup.

### Local Development

```bash
npm install
npm run dev       # Dev server at :5173, proxies /api/* to backend (localhost:8080)
```

### Production Build

```bash
npm run build     # TypeScript + Vite output to dist/
npm run preview   # Preview production build locally
```

## Routing Summary

| Route | Component | Purpose |
|---|---|---|
| `/` | `StatsDashboard` | Real-time DNS interception statistics |
| `/resolvers` | `ResolverList` | Manage upstream DNS resolver endpoints |
| `/rules` | `RuleList` | Manage priority-based DNS redirect rules |
