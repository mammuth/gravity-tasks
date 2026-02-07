# This file is auto-generated from the current state of the database. Instead
# of editing this file, please use the migrations feature of Active Record to
# incrementally modify your database, and then regenerate this schema definition.
#
# This file is the source Rails uses to define your schema when running `bin/rails
# db:schema:load`. When creating a new database, `bin/rails db:schema:load` tends to
# be faster and is potentially less error prone than running all of your
# migrations from scratch. Old migrations may fail to apply correctly if those
# migrations use external dependencies or application code.
#
# It's strongly recommended that you check this file into your version control system.

ActiveRecord::Schema[8.1].define(version: 2026_02_07_110000) do
  create_table "lists", id: :string, force: :cascade do |t|
    t.datetime "created_at", null: false
    t.datetime "deleted_at"
    t.string "name", null: false
    t.float "position", null: false
    t.integer "revision", default: 0, null: false
    t.string "uid", null: false
    t.datetime "updated_at", null: false
    t.index ["uid", "deleted_at"], name: "index_lists_on_uid_and_deleted_at"
    t.index ["uid", "updated_at"], name: "index_lists_on_uid_and_updated_at"
  end

  create_table "tasks", id: :string, force: :cascade do |t|
    t.datetime "archived_at"
    t.datetime "created_at", null: false
    t.datetime "deleted_at"
    t.text "description"
    t.datetime "done_at"
    t.string "list_id"
    t.float "position", null: false
    t.integer "revision", default: 0, null: false
    t.string "status", default: "active", null: false
    t.string "title", null: false
    t.string "uid", null: false
    t.datetime "updated_at", null: false
    t.index ["uid", "deleted_at"], name: "index_tasks_on_uid_and_deleted_at"
    t.index ["uid", "list_id", "status"], name: "index_tasks_on_uid_and_list_id_and_status"
    t.index ["uid", "status"], name: "index_tasks_on_uid_and_status"
    t.index ["uid", "updated_at"], name: "index_tasks_on_uid_and_updated_at"
  end
end
