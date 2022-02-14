# User rules

## admins and mods can read all of a user's fields except passwords
allow_field(user: User, "READ", _other_user: User, field) if
    user.role in [Role::Admin, Role::Moderator] and
    field in ["name", "created_at", "updated_at", "role", "email"];

## users can read all of their own fields except passwords
allow_field(user: User, "READ", other_user: User, field) if
    user.id == other_user.id and
    field in ["name", "created_at", "updated_at", "role", "email"];

## anyone can read ids, names, created_at, and role of other users
allow_field(_, "READ", _other_user: User, field: String) if
    field in ["name", "created_at", "role"];

## admins can change user names or emails
allow_field(user: User, "UPDATE", _other_user: User, field: String) if
    user.role = Role::Admin and
    field in ["name", "email"];

## moderators can do the same but only to other users of role below them
allow_field(user: User, "UPDATE", other_user: User, field: String) if
    user.role = Role::Moderator and
    other_user.role in [Role::Maintainer, Role::Creator, Role::Contributor, Role::Member] and
    field in ["name", "email"];

## admins can delete other users
allow(user: User, "DELETE", _other_user: User) if
    user.role = Role::Admin;

## moderators can also, but again only to other users of role below them
allow(user: User, "DELETE", other_user: User) if
    user.role = Role::Moderator and
    other_user.role in [Role::Maintainer, Role::Creator, Role::Contributor, Role::Member];

# role specific stuff

## admins can assign any role
allow_assign_role(user: User, _role: Role) if
    user.role = Role::Admin;

## Moderators can only assign roles below them
allow_assign_role(user: User, role: Role) if
    user.role = Role::Moderator and
    role in [Role::Maintainer, Role::Creator, Role::Contributor, Role::Member];

## users can update themselves but only on certain fields
allow_field(user: User, "UPDATE", other_user: User, field) if
    field in ["name", "email", "password"] and
    user.id = other_user.id;

## users can delete themselves
allow(user: User, "DELETE", other_user: User) if
    user.id = other_user.id;
