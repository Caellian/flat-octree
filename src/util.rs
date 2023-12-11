//! Utility macros and functions for dealing with octree types.

/// Provides a way to iterate over children tuple by unrolling the provided body
/// 8 times for each.
#[macro_export]
macro_rules! for_each_child {
    ($name: ident: $children: expr => $body: block) => {{
        let children__ = $children;
        {
            let $name = children__.0;
            $body
        }
        {
            let $name = children__.1;
            $body
        }
        {
            let $name = children__.2;
            $body
        }
        {
            let $name = children__.3;
            $body
        }
        {
            let $name = children__.4;
            $body
        }
        {
            let $name = children__.5;
            $body
        }
        {
            let $name = children__.6;
            $body
        }
        {
            let $name = children__.7;
            $body
        }
    }};
}
