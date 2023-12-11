Contributions are welcome in form of Issues and PRs. It's assumed that all
contributed code falls under the same licensing terms as outlined by the project
unless stated otherwise.

There are some things I'd like to point out before you ask or start working on
features though:

- It's impossible to access node children at runtime while retaining wrapper
  node type information as the information is tightly packed and node position
  within the octree is compiled away.

- `DerefMut` can't be implemented for nodes as updating them requires updating
  all sub-nodes in the octree which can't be done via a mutable reference.
