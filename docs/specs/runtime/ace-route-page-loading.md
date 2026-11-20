# ACE route-page loading

The ACE package desktop bridge and Vue project generator consume the AGE
route rows from the app widget. A route such as `"/book/:id" -> use book_detail`
resolves the module alias `book_detail` in the application's package. A Vue
project supplies those modules from `src/front/pages/`. The module's primary
view widget can use a different name. The page and its imported helper modules share the checked package binding. Each routed shell has one `outlet`.

## Navigation and page lifetime

The shell starts at `/`. A `link (to: "...")` or
`sidebar_menu_button (to: "...")` navigates to the destination. A page can
call `router.push(...)` with a literal path or a path assembled from string
segments and integer model fields converted with `.to_string()`.
`router.back()` revisits the preceding history entry.

A page uses the path's parameter bindings for its own `router.param("id")`
reads. Integer fields receive the checked signed 32-bit value. The target
instance runs `Init` when mounted. Its later handlers read the same bindings
and retain that instance's model state while it is mounted.

Navigating to a different path releases the previous instance, including
when two paths use the same page module. Revisiting a path mounts a fresh
instance. The AV event carries the session identity that produced it, and
the Vue outlet keys its component by path. A released page cannot navigate
the active app. Closing and reopening the app resets navigation and creates
a new initial page.

## Diagnostics and consumers

A matched route with a missing module or a module that cannot produce a
checked page presents `ace.route.page`. It names the path or pattern,
module, and failure. A path without a matching declaration presents
`ace.route.no-match`. The shell and valid sibling page links remain usable.

The desktop entry is `NodeDesktopBridge::load_package`. The Vue product
uses `generate_routed_package_sfcs` to publish the shell, page SFCs and their
route mapping. The AutoUI Vue tool uses this product when
`AUTO_UI_ACE_VUE=1` and a routed front is present. A project without routes
continues through its existing entry path.

The [VM page loading contract](vm-route-page-loading.md) records the
corresponding VM behavior. The [Reader](../../../examples/ui/018-book-reader)
and [Kanban](../../../examples/ui/022-kanban) projects use this routing
capability within PLAN-105's whole-app integration, including their store,
backend and list dependencies.
