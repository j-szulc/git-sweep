# Git sweep

Simple tool to clean up your disk from unused git repositories.
The repositories are checked for the following conditions:
- Are there any new/modified and not ignored uncommited files
- Are there any unpushed commits (after updating the remote)

Note that the script uses your git agent to check what is the remote status, so it will not work offline.

## Disclaimer

You use the script entirely at your own risk.
You are ultimately responsible to decide whether the selected folders are safe to delete.

```bash
$ git-sweep ~/repos/*
❌ /home/user/repos/repo1 Dirty local index
❌ /home/user/repos/repo2 Dirty local index
❌ /home/user/repos/repo3 Dirty local index
❌ /home/user/repos/repo4 Dirty local index, Ahead of origin
✅ /home/user/repos/repo5
✅ /home/user/repos/repo6
✅ /home/user/repos/repo7
❌ /home/user/repos/repo8 Dirty local index, Ahead of upstream
❌ /home/user/repos/repo9 Dirty local index
❌ /home/user/repos/repo10 Dirty local index, Ahead of origin
❌ /home/user/repos/repo11 Dirty local index
❌ /home/user/repos/repo12 Dirty local index, Error: Remote HEAD not found
❌ /home/user/repos/repo13 Dirty local index, Ahead of upstream
❌ /home/user/repos/repo14 Dirty local index
❌ /home/user/repos/repo15 Dirty local index
❌ /home/user/repos/repo16 Dirty local index, Ahead of origin
❌ /home/user/repos/repo17 Dirty local index
❌ /home/user/repos/repo18 Error: Local commit is neither ahead nor behind remote!
❌ /home/user/repos/repo19 Dirty local index
✅ /home/user/repos/repo20 
✅ /home/user/repos/repo21 
❌ /home/user/repos/repo22 Dirty local index, Error: Local commit is neither ahead nor behind remote!
❌ /home/user/repos/repo23 Dirty local index, Error: Local commit is neither ahead nor behind remote!
? Select repos to delete  
> [ ] /home/user/repos/repo5
  [ ] /home/user/repos/repo6
  [ ] /home/user/repos/repo7
  [ ] /home/user/repos/repo20
  [ ] /home/user/repos/repo21

```