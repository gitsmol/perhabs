# Perhabs Changelog
## 2023-05-16
Added spatial drawing puzzles.

## 2023-04-09
Added a main menu. Split tools and sessions.

## 2023-02-25
- Built a new way to load app data and excercise data. All loading is done by asset_loader with priority:
    - find on disk
    - find on web
    - fallback to hardcoded default
- Refactored sequences to use asset_loader.
