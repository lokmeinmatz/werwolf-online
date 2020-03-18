# Database Structure

## Users

### Fields
- user_name: text !null unique
- session_id: text (8 chars long uppercase alphabetic / digits)
- role: text
- joined: int UNIX_TIME
- state: text

### Thoughts
A user can only be in one session at a time