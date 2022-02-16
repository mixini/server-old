# User rules

## admins and mods can read all of a user's fields except passwords
allow_field(user: User, _: Read, _other_user: User, field) if
    user.role in [Role::Admin, Role::Moderator] and
    field in ["created_at", "updated_at", "name", "email", "role"];

## users can read all of their own fields except passwords
allow_field(user: User, _: Read, other_user: User, field) if
    user.id == other_user.id and
    field in ["created_at", "updated_at", "name", "email", "role"];

## anyone can read names, created_at, and role of other users
allow_field(_, _: Read, _other_user: User, field: String) if
    field in ["created_at", "name", "role"];

## admins can change everything for a user except the password
allow(user: User, update: UpdateUser, _other_user: User) if
    user.role = Role::Admin and
    update.password = nil;

## moderators can do the same but only to other users of role below them
## they cannot assign roles higher than or equal to themselves
allow(user: User, update: UpdateUser, other_user: User) if
    user.role = Role::Moderator and
    other_user.role in [Role::Maintainer, Role::Creator, Role::Contributor, Role::Member] and
    update.role in [Role::Maintainer, Role::Creator, Role::Contributor, Role::Member, nil] and
    update.password = nil;

## users can update themselves but not their role
allow(user: User, update: UpdateUser, other_user: User) if
    user.id = other_user.id and
    changes.role = nil;

## admins can delete other users
allow(user: User, _: Delete, _other_user: User) if
    user.role = Role::Admin;

## moderators can also, but again only to other users of role below them
allow(user: User, _: Delete, other_user: User) if
    user.role = Role::Moderator and
    other_user.role in [Role::Maintainer, Role::Creator, Role::Contributor, Role::Member];

## users can delete themselves
allow(user: User, _: Delete, other_user: User) if
    user.id = other_user.id;
