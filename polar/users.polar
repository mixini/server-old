# User rules

## anyone read names, created_at, and role of other users
allow_field(_, "READ", _other_user: User, field: String) if
    field in ["name", "created_at", "role"];

## admins can change user names or emails
allow_field(user: User, "UPDATE", _other_user: User, field: String) if
    user.role = "Administrator" and
    field in ["name", "email"];

## moderators can do the same but only to other users of role below them
allow_field(user: User, "UPDATE", other_user: User, field: String) if
    user.role = "Moderator" and
    other_user.role in ["Maintainer", "Creator", "Contributor", "Member"] and
    field in ["name", "email"];

## admins can delete other users
allow(user: User, "DELETE", _other_user: User) if
    user.role = "Administrator";

## moderators can also, but again only to other users of role below them
allow(user: User, "DELETE", other_user: User) if
    user.role = "Moderator" and
    other_user.role in ["Maintainer", "Creator", "Contributor", "Member"];

# role specific stuff

## admins can assign any role
allow_assign_role(user: User, _role: Role) if
    user.role = "Administrator";

## Moderators can only assign roles below them
allow_assign_role(user: User, role: Role) if
    user.role = "Moderator" and
    role in ["Maintainer", "Creator", "Contributor", "Member"];

## users can update themselves but only on certain fields
allow_field(user: User, action: String, other_user: User, field) if
    field in ["name", "email", "password"] and
    user.id = other_user.id and
    action in ["UPDATE"];

## users can delete themselves
allow(user: User, "DELETE", other_user: User) if
    user.id = other_user.id;
