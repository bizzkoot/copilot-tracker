# AGENT NOTES

## Communication Requirement
**CRITICAL: ALWAYS USE ASK_USER TOOL**
**CRITICAL: REPLACE ALL TOOL `question` WITH `ask_user` TOOL**

**MUST use `ask_user` tool for:**

- ANY reply to the user
- ANY response to the user
- ANY summary for the user

**Session ending: ONLY end the session when the user EXPLICITLY requests it through the "ask_user" tool.**

**NO EXCEPTIONS. This rule is MANDATORY and NON-NEGOTIABLE.**

## ask_user Tool Retry Behavior

**When ask_user returns `shouldRetry: true`:**
1. **IMMEDIATELY retry** the ask_user tool call with the SAME parameters
2. Do NOT summarize, end the session, or ask the user what happened
3. This is an automatic UI recovery mechanism - the question failed to display
4. The retry happens within the same session (no premium request cost)

**When ask_user returns `shouldRetry: false` and `timedOut: true`:**
1. The user did not respond within 5 minutes
2. Ask the user if they want to continue or if they missed the question
3. You may re-ask the question if the user confirms they want to continue

**When ask_user returns `shouldRetry: false` with error:**
1. Maximum retry attempts (3) have been exceeded
2. Inform the user: "There was a technical issue displaying the question. Please try again."
3. Ask if they want to continue with the task
