# Swedish Decimal Time Format Feature

## Overview

This document describes the enhancements made to the Chronos Log application to support Swedish decimal time format with comma as the decimal separator.

## Changes Made

### 1. New Function: `format_minutes_to_decimal`

**Location:** `src/database.rs`

A new public function has been added to convert minutes to decimal hours using the Swedish comma separator:

```rust
pub fn format_minutes_to_decimal(total_minutes: i32) -> String {
    let hours = total_minutes as f64 / 60.0;
    // Format with 2 decimal places and replace . with ,
    format!("{:.2}", hours).replace('.', ",")
}
```

**Examples:**
- 30 minutes → "0,50" (half hour)
- 60 minutes → "1,00" (one hour)
- 90 minutes → "1,50" (one and a half hours)
- 480 minutes → "8,00" (eight hours)
- 15 minutes → "0,25" (quarter hour)
- 45 minutes → "0,75" (three quarters hour)

### 2. UI Updates

**Location:** `src/ui.rs`

The user interface has been updated to display time in both formats simultaneously:

#### Time Tracking View
- Each time entry now shows both HH:MM format and decimal format
- Example: `02:30 (2,50h)`
- The decimal format is displayed in a lighter gray color for visual distinction

#### Daily Summary View
- Activity summaries show both formats
- Total for the day shows both formats
- The "Copy" button now copies the decimal format (with comma) instead of HH:MM format
- This makes it easier to paste into Swedish time management systems

#### Total Time Display
- Both the Time Tracking and Daily Summary views show the total in both formats
- The decimal format appears next to the HH:MM format in a slightly smaller font
- Colors are preserved for the 8-hour threshold warnings (red < 8h, green = 8h, orange > 8h)

### 3. Testing

**Location:** `src/database.rs` (tests module)

A comprehensive test suite has been added for the new function:

```rust
#[test]
fn test_format_minutes_to_decimal() {
    assert_eq!(format_minutes_to_decimal(30), "0,50");
    assert_eq!(format_minutes_to_decimal(60), "1,00");
    assert_eq!(format_minutes_to_decimal(90), "1,50");
    assert_eq!(format_minutes_to_decimal(150), "2,50");
    assert_eq!(format_minutes_to_decimal(480), "8,00");
    assert_eq!(format_minutes_to_decimal(15), "0,25");
    assert_eq!(format_minutes_to_decimal(45), "0,75");
}
```

All tests pass successfully.

## Benefits for Swedish Users

1. **Compliance with Swedish Standards:** Uses comma (,) as the decimal separator, which is the Swedish/European standard
2. **Easy Copy-Paste:** The copy button in the Daily Summary view now copies the decimal format, making it easy to paste into Swedish time reporting systems
3. **Dual Display:** Both formats are shown simultaneously, so users can verify their entries in the format they prefer
4. **No Breaking Changes:** The existing HH:MM input and display functionality remains unchanged

## Time Input

Time input continues to use the HH:MM format (e.g., `02:30` for 2 hours and 30 minutes). This is intuitive and widely understood.

## Example Usage

When you log 2 hours and 30 minutes (entered as `02:30`), the application will display:
- In the entry list: `02:30 (2,50h)`
- In the daily summary: `02:30 (2,50h)`
- When copying: `2,50` (ready to paste into your time management system)

## Building and Running

```bash
# Build in release mode
cargo build --release

# Run the application
cargo run --release

# Run tests
cargo test
```

## Compatibility

- All existing functionality remains unchanged
- Database format is unchanged (still stores minutes as integers)
- The change is purely presentational in the UI layer