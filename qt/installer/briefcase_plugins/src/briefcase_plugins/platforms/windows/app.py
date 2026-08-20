# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

from __future__ import annotations

from briefcase.platforms.windows.app import (
    WindowsAppBuildCommand as _WindowsAppBuildCommand,
    WindowsAppCreateCommand as _WindowsAppCreateCommand,
    WindowsAppDevCommand as _WindowsAppDevCommand,
    WindowsAppMixin as _WindowsAppMixin,
    WindowsAppOpenCommand as _WindowsAppOpenCommand,
    WindowsAppPackageCommand as _WindowsAppPackageCommand,
    WindowsAppPublishCommand as _WindowsAppPublishCommand,
    WindowsAppRunCommand as _WindowsAppRunCommand,
    WindowsAppUpdateCommand as _WindowsAppUpdateCommand,
)


class WindowsAppMixin(_WindowsAppMixin):
    def bundle_package_executable_path(self, app):
        return "AnkiStudyLab.exe"

    def distribution_filename(self, app):
        suffix = "zip" if app.packaging_format == "zip" else "msi"
        return f"AnkiStudyLab-{app.version}.{suffix}"


class WindowsAppCreateCommand(WindowsAppMixin, _WindowsAppCreateCommand):
    pass


class WindowsAppUpdateCommand(WindowsAppMixin, _WindowsAppUpdateCommand):
    pass


class WindowsAppBuildCommand(WindowsAppMixin, _WindowsAppBuildCommand):
    pass


class WindowsAppOpenCommand(WindowsAppMixin, _WindowsAppOpenCommand):
    pass


class WindowsAppDevCommand(WindowsAppMixin, _WindowsAppDevCommand):
    pass


class WindowsAppRunCommand(WindowsAppMixin, _WindowsAppRunCommand):
    pass


class WindowsAppPackageCommand(WindowsAppMixin, _WindowsAppPackageCommand):
    pass


class WindowsAppPublishCommand(WindowsAppMixin, _WindowsAppPublishCommand):
    pass


# Declare the briefcase command bindings
create = WindowsAppCreateCommand
update = WindowsAppUpdateCommand
build = WindowsAppBuildCommand
open = WindowsAppOpenCommand
dev = WindowsAppDevCommand
run = WindowsAppRunCommand
package = WindowsAppPackageCommand
publish = WindowsAppPublishCommand
